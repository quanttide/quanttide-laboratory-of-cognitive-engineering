use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use qtcloud_think::ai::AiClient;
use qtcloud_think::config::Config;
use qtcloud_think::db::Database;
use qtcloud_think::error::{Result, ThinkCloudError};
use qtcloud_think::models::*;

#[derive(Parser)]
#[command(name = "qtcloud-think", about = "收集念头，交互产生想法")]
struct Cli {
    /// Active session ID (default: most recent)
    #[arg(short, long)]
    session: Option<i64>,

    /// Material file path
    #[arg(short, long)]
    material: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 收集一条念头，自动按主题分类并生成想法
    Collect {
        /// 念头内容，如 "复现步骤：缺少环境变量"
        text: String,
    },
    /// 处理所有待处理的念头
    Process,
    /// 列出当前会话的想法
    Ideas,
    /// 接受一个想法
    Accept {
        /// 想法 ID
        id: i64,
    },
    /// 拒绝一个想法
    Reject {
        /// 想法 ID
        id: i64,
    },
    /// 管理会话
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    /// 列出可用的念头模板
    Templates,
    /// 查看当前会话状态
    Status,
    /// 导出会话数据为 JSON
    Export,
}

#[derive(Subcommand)]
enum SessionAction {
    /// Create a new session
    New {
        /// Session title
        #[arg(long)]
        title: Option<String>,
    },
    /// List all sessions
    List,
    /// Switch to a session
    Switch {
        /// Session ID
        id: i64,
    },
}

struct App {
    db: Database,
    config: Config,
    current_session: Session,
}

impl App {
    fn new(db: Database, config: Config, session_id: Option<i64>) -> Result<Self> {
        let current_session = if let Some(id) = session_id {
            db.get_session(id)?
                .ok_or_else(|| ThinkCloudError::Other(format!("Session {id} not found")))?
        } else {
            let sessions = db.list_sessions()?;
            sessions.into_iter().next().unwrap_or_else(|| {
                db.create_session(Some("默认会话")).expect("Failed to create default session")
            })
        };
        Ok(Self {
            db,
            config,
            current_session,
        })
    }

    fn switch_session(&mut self, id: i64) -> Result<()> {
        let session = self.db.get_session(id)?
            .ok_or_else(|| ThinkCloudError::Other(format!("Session {id} not found")))?;
        self.current_session = session;
        Ok(())
    }

    fn submit_thought(&mut self, text: &str) -> Result<()> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ThinkCloudError::Other("Thought cannot be empty".into()));
        }

        let material_id = None;
        let thought = self.db.create_thought(self.current_session.id, material_id, text)?;
        println!("✓ 念头已提交 #{}\n  内容: {}", thought.id, thought.content);
        Ok(())
    }

    fn extract_category(text: &str) -> String {
        if let Some(pos) = text.find('：') {
            let cat = text[..pos].trim();
            if !cat.is_empty() {
                return cat.to_string();
            }
        }
        "通用".into()
    }

    fn process_thoughts(&self) -> Result<()> {
        let pending = self.db.get_pending_thoughts_count(self.current_session.id)?;
        if pending == 0 {
            println!("没有待处理的念头");
            return Ok(());
        }

        let api_key = self.config.api_key()
            .ok_or_else(|| ThinkCloudError::Other(
                "DEEPSEEK_API_KEY 环境变量未设置".into()
            ))?;
        let client = AiClient::new(api_key, self.config.ai.base_url.clone(), self.config.ai.model.clone());

        let materials = self.db.get_session_materials(self.current_session.id)?;
        let accepted_ideas = self.db.get_accepted_ideas(self.current_session.id)?;
        let all_thoughts = self.db.get_recent_thoughts(
            self.current_session.id,
            self.config.ui.thought_window,
        )?;

        // Group pending thoughts by category
        let pending_thoughts: Vec<&Thought> = all_thoughts.iter()
            .filter(|t| matches!(t.status, ThoughtStatus::Pending | ThoughtStatus::Processing))
            .collect();

        let mut groups: std::collections::BTreeMap<String, Vec<&Thought>> = std::collections::BTreeMap::new();
        for t in &pending_thoughts {
            let cat = Self::extract_category(&t.content);
            groups.entry(cat).or_default().push(t);
        }

        println!("⟳ AI 处理中（{} 个主题）...", groups.len());

        let mut any_failed = false;
        for (category, thoughts) in &groups {
            let ctx = AiContext {
                materials: materials.clone(),
                thoughts: thoughts.iter().map(|t| (*t).clone()).collect(),
                accepted_ideas: accepted_ideas.clone(),
                max_tokens: self.config.ui.max_context_tokens,
            };
            let mut ctx = ctx;
            client.truncate_context(&mut ctx);

            println!("  [{category}] AI 处理中（{} 条念头）...", thoughts.len());
            match client.call(&ctx) {
                Ok(content) => {
                    let idea = self.db.create_idea(self.current_session.id, &content)?;
                    let thought_ids: Vec<i64> = thoughts.iter().map(|t| t.id).collect();
                    self.db.link_idea_to_thoughts(idea.id, &thought_ids)?;
                    for tid in &thought_ids {
                        self.db.update_thought_status(*tid, &ThoughtStatus::Completed)?;
                    }
                    println!("\n💡 [{category}] 想法 #{}\n{}", idea.id, idea.content);
                    println!("\n   qtcloud-think accept {}  — 接受", idea.id);
                    println!("   qtcloud-think reject {}   — 拒绝", idea.id);
                }
                Err(e) => {
                    any_failed = true;
                    for t in thoughts {
                        self.db.update_thought_status(t.id, &ThoughtStatus::Failed)?;
                    }
                    eprintln!("  [{category}] ✗ AI 处理失败: {e}");
                }
            }
        }

        if any_failed {
            Err(ThinkCloudError::Other("部分念头处理失败".into()))
        } else {
            Ok(())
        }
    }

    fn list_ideas(&self) -> Result<()> {
        let ideas = self.db.get_recent_ideas(self.current_session.id, 20)?;
        if ideas.is_empty() {
            println!("当前会话没有想法");
            return Ok(());
        }

        for idea in &ideas {
            let status_char = match idea.status {
                IdeaStatus::Pending => "○",
                IdeaStatus::Accepted => "✓",
                IdeaStatus::Rejected => "✗",
                IdeaStatus::Failed => "⚠",
            };
            println!(" [#{}] {}  {}", idea.id, status_char, idea.content);
        }
        Ok(())
    }

    fn accept_idea(&self, id: i64) -> Result<()> {
        self.db.update_idea_status(id, &IdeaStatus::Accepted)?;
        println!("✓ 想法 #{id} 已接受");
        Ok(())
    }

    fn reject_idea(&self, id: i64) -> Result<()> {
        self.db.update_idea_status(id, &IdeaStatus::Rejected)?;
        println!("✓ 想法 #{id} 已拒绝");
        Ok(())
    }

    fn show_templates(&self) {
        println!("可用念头模板：");
        for (i, t) in self.config.ui.thought_templates.iter().enumerate() {
            println!("  {}. {}", i + 1, t);
        }
        println!("\n用法: qtcloud-think collect \"模板文本 + 你的内容\"");
    }

    fn show_status(&self) -> Result<()> {
        let session = &self.current_session;
        let thought_count = self.db.get_recent_thoughts(session.id, 10000).unwrap_or_default().len();
        let idea_count = self.db.get_recent_ideas(session.id, 10000).unwrap_or_default().len();
        let pending = self.db.get_pending_thoughts_count(session.id)?;
        let materials = self.db.get_session_materials(session.id)?;

        println!("会话 #{}", session.id);
        println!("  标题: {}", session.title.as_deref().unwrap_or("未命名"));
        println!("  材料: {} 个", materials.len());
        for m in &materials {
            println!("    - {}", m.path.as_deref().unwrap_or("unnamed"));
        }
        println!("  念头: {} 条（{} 条待处理）", thought_count, pending);
        println!("  想法: {} 条", idea_count);
        Ok(())
    }

    fn export_json(&self) -> Result<()> {
        let session = &self.current_session;
        let thoughts = self.db.get_recent_thoughts(session.id, 10000).unwrap_or_default();
        let ideas = self.db.get_recent_ideas(session.id, 10000).unwrap_or_default();
        let materials = self.db.get_session_materials(session.id).unwrap_or_default();

        let export = serde_json::json!({
            "session": {
                "id": session.id,
                "title": session.title,
            },
            "thoughts": thoughts,
            "ideas": ideas,
            "materials": materials,
        });

        let json_str = serde_json::to_string_pretty(&export).unwrap_or_default();
        let path = format!("qtcloud_session_{}.json", session.id);
        std::fs::write(&path, &json_str)?;
        println!("✓ 已导出到 {path}");
        Ok(())
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
        .init();

    let config = Config::load()?;
    let db_path = config.storage.data_dir.join("thinkcloud.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db = Database::open(&db_path)?;
    let cli = Cli::parse();

    // Handle material flag globally — load before command
    if let Some(ref path) = cli.material {
        let material = db.create_material(Some(path), None)?;
        let sessions = db.list_sessions()?;
        if let Some(session) = sessions.first() {
            db.add_material_to_session(session.id, material.id)?;
        }
    }

    let mut app = App::new(db, config, cli.session)?;

    match cli.command {
        Commands::Collect { text } => {
            app.submit_thought(&text)?;
            app.process_thoughts()?;
        }
        Commands::Process => {
            app.process_thoughts()?;
        }
        Commands::Ideas => {
            app.list_ideas()?;
        }
        Commands::Accept { id } => {
            app.accept_idea(id)?;
        }
        Commands::Reject { id } => {
            app.reject_idea(id)?;
        }
        Commands::Session { action } => match action {
            SessionAction::New { title } => {
                let session = app.db.create_session(title.as_deref())?;
                println!("✓ 已创建会话 #{}", session.id);
                println!("  使用 qtcloud-think -s {} 切换到此会话", session.id);
            }
            SessionAction::List => {
                let sessions = app.db.list_sessions()?;
                for s in &sessions {
                    let marker = if s.id == app.current_session.id { " ◀" } else { "" };
                    println!("  #{}  {}{}", s.id, s.title.as_deref().unwrap_or("未命名"), marker);
                }
            }
            SessionAction::Switch { id } => {
                app.switch_session(id)?;
                println!("✓ 已切换到会话 #{id}");
            }
        },
        Commands::Templates => {
            app.show_templates();
        }
        Commands::Status => {
            app.show_status()?;
        }
        Commands::Export => {
            app.export_json()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use qtcloud_think::db::Database;

    fn setup_app() -> App {
        let db = Database::open_in_memory().unwrap();
        let config = Config::default();
        App::new(db, config, None).unwrap()
    }

    #[test]
    fn test_app_creates_default_session() {
        let app = setup_app();
        assert!(app.current_session.id > 0);
    }

    #[test]
    fn test_submit_thought() {
        let mut app = setup_app();
        app.submit_thought("复现步骤：缺少环境变量").unwrap();
        let thoughts = app.db.get_recent_thoughts(app.current_session.id, 10).unwrap();
        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0].content, "复现步骤：缺少环境变量");
    }

    #[test]
    fn test_submit_empty_thought_fails() {
        let mut app = setup_app();
        let result = app.submit_thought("  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_accept_idea() {
        let app = setup_app();
        let idea = app.db.create_idea(app.current_session.id, "test idea").unwrap();
        app.accept_idea(idea.id).unwrap();
        let accepted = app.db.get_accepted_ideas(app.current_session.id).unwrap();
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].status, IdeaStatus::Accepted);
    }

    #[test]
    fn test_reject_idea() {
        let app = setup_app();
        let idea = app.db.create_idea(app.current_session.id, "test idea").unwrap();
        app.reject_idea(idea.id).unwrap();
    }

    #[test]
    fn test_switch_session() {
        let mut app = setup_app();
        let s1 = app.current_session.id;
        let s2 = app.db.create_session(Some("session 2")).unwrap();

        app.switch_session(s2.id).unwrap();
        assert_eq!(app.current_session.id, s2.id);

        app.switch_session(s1).unwrap();
        assert_eq!(app.current_session.id, s1);
    }

    #[test]
    fn test_switch_session_not_found() {
        let mut app = setup_app();
        let result = app.switch_session(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_ideas_empty() {
        let app = setup_app();
        // Should not crash
        app.list_ideas().unwrap();
    }

    #[test]
    fn test_show_status() {
        let app = setup_app();
        app.show_status().unwrap();
    }

    #[test]
    fn test_extract_category() {
        assert_eq!(App::extract_category("复现步骤：缺少环境变量"), "复现步骤");
        assert_eq!(App::extract_category("根因分析：时区问题"), "根因分析");
        assert_eq!(App::extract_category("我想引入资产价值分级"), "通用");
        assert_eq!(App::extract_category("：只有冒号"), "通用");
        assert_eq!(App::extract_category(""), "通用");
    }

    #[test]
    fn test_process_no_pending_thoughts() {
        let app = setup_app();
        app.process_thoughts().unwrap();
    }

    #[test]
    fn test_extract_category_groups_thoughts() {
        let thoughts = [
            "复现步骤：缺少环境变量",
            "根因分析：时区问题",
            "我想引入资产价值分级",
        ];
        let categories: Vec<String> = thoughts.iter().map(|t| App::extract_category(t)).collect();
        assert_eq!(categories[0], "复现步骤");
        assert_eq!(categories[1], "根因分析");
        assert_eq!(categories[2], "通用");
        // All different categories — no over-correlation
        let unique: std::collections::HashSet<&str> =
            categories.iter().map(|s| s.as_str()).collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn test_export_json() {
        let app = setup_app();
        app.export_json().unwrap();
        // Clean up
        let path = format!("qtcloud_session_{}.json", app.current_session.id);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_cli_missing_session() {
        let db = Database::open_in_memory().unwrap();
        let config = Config::default();
        let result = App::new(db, config, Some(999));
        assert!(result.is_err());
    }
}
