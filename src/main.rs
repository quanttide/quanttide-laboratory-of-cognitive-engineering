use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use thinkcloud::ai::AiClient;
use thinkcloud::config::Config;
use thinkcloud::db::Database;
use thinkcloud::error::Result;
use thinkcloud::models::*;
use thinkcloud::ui::{self, AppStatus, FocusArea, UiState};

enum AppEvent {
    AiResult(std::result::Result<String, String>),
}

struct App {
    db: Database,
    config: Config,
    state: UiState,
    ai_tx: Option<mpsc::Sender<i64>>,
    ai_rx: mpsc::Receiver<AppEvent>,
    cursor_position: usize,
}

impl App {
    fn new(db: Database, config: Config, ai_rx: mpsc::Receiver<AppEvent>) -> Result<Self> {
        let mut state = UiState::empty();
        let sessions = db.list_sessions()?;

        state.templates = config.ui.thought_templates.clone();

        let initial_session = sessions.first().cloned();
        if let Some(ref session) = initial_session {
            let thoughts = db.get_recent_thoughts(session.id, config.ui.thought_window)?;
            let materials = db.get_session_materials(session.id)?;
            let current_idea = db.get_latest_pending_idea(session.id)?;
            let status = if session.ai_pending {
                AppStatus::Processing
            } else {
                AppStatus::Normal
            };
            state.thoughts = thoughts;
            state.materials = materials;
            state.current_idea = current_idea;
            state.sessions = sessions;
            state.current_session = initial_session;
            state.status = status;
        } else {
            let session = db.create_session(Some("默认会话"))?;
            state.sessions = vec![session.clone()];
            state.current_session = Some(session);
        }

        Ok(Self {
            db,
            config,
            state,
            ai_tx: None,
            ai_rx,
            cursor_position: 0,
        })
    }

    fn set_ai_channel(&mut self, tx: mpsc::Sender<i64>) {
        self.ai_tx = Some(tx);
    }

    fn trigger_ai(&mut self) -> Result<()> {
        let session_id = match &self.state.current_session {
            Some(s) => s.id,
            None => return Ok(()),
        };

        // P0: Serial queue — check if AI is already pending
        if self.state.status == AppStatus::Processing {
            tracing::debug!("AI already processing for session {session_id}, skipping");
            return Ok(());
        }

        // Check for pending thoughts
        let pending_count = self.db.get_pending_thoughts_count(session_id)?;
        if pending_count == 0 {
            return Ok(());
        }

        // Mark session as pending
        self.db.set_session_ai_pending(session_id, true)?;
        self.state.status = AppStatus::Processing;

        // Send to AI task
        if let Some(tx) = &self.ai_tx {
            let _ = tx.try_send(session_id);
        }

        Ok(())
    }

    fn submit_thought(&mut self, content: &str) -> Result<()> {
        let content = content.trim();
        if content.is_empty() {
            return Ok(());
        }

        let session_id = match &self.state.current_session {
            Some(s) => s.id,
            None => return Ok(()),
        };

        // Handle commands
        if content.starts_with(':') {
            return self.handle_command(content);
        }

        // Save thought to DB
        let thought = self.db.create_thought(session_id, None, content)?;
        self.state.thoughts.push(thought);
        self.state.input.clear();
        self.cursor_position = 0;

        // Trim thought list to window size
        let window = self.config.ui.thought_window;
        if self.state.thoughts.len() > window {
            self.state.thoughts.drain(0..self.state.thoughts.len() - window);
        }

        // P0: Serial queue — trigger AI processing
        self.trigger_ai()?;

        Ok(())
    }

    fn handle_command(&mut self, cmd: &str) -> Result<()> {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        match parts[0] {
            ":material" | ":m" => {
                if let Some(path) = parts.get(1) {
                    self.load_material(path)?;
                }
            }
            ":session" => {
                if let Some(sub) = parts.get(1) {
                    match *sub {
                        "new" => {
                            let session = self.db.create_session(None)?;
                            self.state.sessions.push(session.clone());
                            self.switch_session(session.id)?;
                        }
                        "switch" => {
                            if let Some(id_str) = parts.get(2) {
                                if let Ok(id) = id_str.parse::<i64>() {
                                    self.switch_session(id)?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            ":export" => {
                // P2: Export support
                self.export_json()?;
            }
            ":help" => {
                // Show help would go here
            }
            ":quit" | ":q" => {
                // Handled at event loop level
            }
            _ => {}
        }
        self.state.input.clear();
        self.cursor_position = 0;
        Ok(())
    }

    fn load_material(&mut self, path: &str) -> Result<()> {
        let path_buf = PathBuf::from(path);
        let snippet = if path_buf.exists() {
            std::fs::read_to_string(&path_buf)
                .ok()
                .map(|s| s.chars().take(200).collect::<String>())
        } else {
            None
        };

        let material = self.db.create_material(Some(path), snippet.as_deref())?;
        if let Some(session) = &self.state.current_session {
            self.db.add_material_to_session(session.id, material.id)?;
        }
        self.state.materials.push(material);
        Ok(())
    }

    fn switch_session(&mut self, session_id: i64) -> Result<()> {
        let session = match self.db.get_session(session_id)? {
            Some(s) => s,
            None => return Ok(()),
        };
        let thoughts = self.db.get_recent_thoughts(session_id, self.config.ui.thought_window)?;
        let materials = self.db.get_session_materials(session_id)?;
        let current_idea = self.db.get_latest_pending_idea(session_id)?;

        let status = if session.ai_pending {
            AppStatus::Processing
        } else {
            AppStatus::Normal
        };

        self.state.thoughts = thoughts;
        self.state.materials = materials;
        self.state.current_idea = current_idea;
        self.state.current_session = Some(session);
        self.state.status = status;
        self.state.scroll_offset = 0;
        Ok(())
    }

    fn accept_idea(&mut self) -> Result<()> {
        if let Some(idea) = &self.state.current_idea {
            self.db.update_idea_status(idea.id, &IdeaStatus::Accepted)?;
            self.state.current_idea = None;
            self.state.status = AppStatus::Normal;
        }
        Ok(())
    }

    fn reject_idea(&mut self) -> Result<()> {
        if let Some(idea) = &self.state.current_idea {
            self.db.update_idea_status(idea.id, &IdeaStatus::Rejected)?;
            self.state.current_idea = None;
            self.state.status = AppStatus::Normal;
        }
        Ok(())
    }

    fn retry_ai(&mut self) -> Result<()> {
        if let AppStatus::Error(_) = &self.state.status {
            self.state.status = AppStatus::Normal;
            // Mark any failed thoughts as pending again
            if let Some(_session) = &self.state.current_session {
                for thought in &mut self.state.thoughts {
                    if thought.status == ThoughtStatus::Failed {
                        self.db.update_thought_status(thought.id, &ThoughtStatus::Pending)?;
                        thought.status = ThoughtStatus::Pending;
                    }
                }
            }
            self.trigger_ai()?;
        }
        Ok(())
    }

    fn handle_ai_result(&mut self, result: std::result::Result<String, String>) -> Result<()> {
        let session_id = match &self.state.current_session {
            Some(s) => s.id,
            None => return Ok(()),
        };

        // Clear AI pending flag
        self.db.set_session_ai_pending(session_id, false)?;

        match result {
            Ok(content) => {
                // Create the idea and link to pending thoughts
                let idea = self.db.create_idea(session_id, &content)?;

                // Find thought IDs that were pending and link them
                let thought_ids: Vec<i64> = self
                    .state
                    .thoughts
                    .iter()
                    .filter(|t| matches!(t.status, ThoughtStatus::Pending | ThoughtStatus::Processing))
                    .map(|t| t.id)
                    .collect();

                if !thought_ids.is_empty() {
                    self.db.link_idea_to_thoughts(idea.id, &thought_ids)?;
                    // Mark those thoughts as completed
                    for tid in &thought_ids {
                        self.db.update_thought_status(*tid, &ThoughtStatus::Completed)?;
                    }
                    for t in &mut self.state.thoughts {
                        if thought_ids.contains(&t.id) {
                            t.status = ThoughtStatus::Completed;
                        }
                    }
                }

                self.state.current_idea = Some(idea);
                self.state.status = AppStatus::Normal;

                // P0: Serial queue drain — check if more pending thoughts came in while processing
                if self.db.get_pending_thoughts_count(session_id)? > 0 {
                    self.trigger_ai()?;
                }
            }
            Err(e) => {
                tracing::error!("AI processing failed: {e}");
                // Mark pending thoughts as failed
                for t in &mut self.state.thoughts {
                    if t.status == ThoughtStatus::Pending || t.status == ThoughtStatus::Processing {
                        self.db.update_thought_status(t.id, &ThoughtStatus::Failed)?;
                        t.status = ThoughtStatus::Failed;
                    }
                }
                self.state.status = AppStatus::Error(e);
            }
        }

        Ok(())
    }

    fn export_json(&self) -> Result<()> {
        if let Some(session) = &self.state.current_session {
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
            let export_path = PathBuf::from(format!(
                "thinkcloud_export_session_{}.json",
                session.id
            ));
            std::fs::write(&export_path, json_str).ok();
        }
        Ok(())
    }
}

async fn ai_worker(
    db_path: PathBuf,
    config: Config,
    mut rx: mpsc::Receiver<i64>,
    result_tx: mpsc::Sender<AppEvent>,
) {
    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("Failed to open database in AI worker: {e}");
            return;
        }
    };

    let Some(api_key) = config.api_key() else {
        tracing::error!("No API key configured for AI worker");
        return;
    };

    let client = AiClient::new(api_key, config.ai.base_url.clone(), config.ai.model.clone());

    while let Some(session_id) = rx.recv().await {
        tracing::info!("AI worker processing session {session_id}");

        // Build context
        let ctx = match db.get_ai_context(session_id, config.ui.thought_window, config.ui.max_context_tokens) {
            Ok(ctx) => ctx,
            Err(e) => {
                let _ = result_tx.send(AppEvent::AiResult(Err(format!("Context build failed: {e}")))).await;
                continue;
            }
        };

        // Truncate if needed (P2: token budget)
        let mut ctx = ctx;
        client.truncate_context(&mut ctx);

        // Call AI
        let result = client.call(&ctx).await;
        let event = match result {
            Ok(content) => AppEvent::AiResult(Ok(content)),
            Err(e) => AppEvent::AiResult(Err(format!("{e}"))),
        };
        let _ = result_tx.send(event).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
        .init();

    let config = Config::load()?;
    let db_path = config.storage.data_dir.join("thinkcloud.db");

    // Ensure data directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db = Database::open(&db_path)?;

    // Use default session if none exists
    let sessions = db.list_sessions()?;
    if sessions.is_empty() {
        db.create_session(Some("默认会话"))?;
    }

    // Channels for AI worker communication
    let (ai_tx, ai_rx_channel) = mpsc::channel::<i64>(32);
    let (result_tx, result_rx) = mpsc::channel::<AppEvent>(32);

    // Spawn AI worker
    let worker_db_path = db_path.clone();
    let worker_config = config.clone();
    tokio::spawn(async move {
        ai_worker(worker_db_path, worker_config, ai_rx_channel, result_tx).await;
    });

    // Initialize app
    let mut app = App::new(db, config, result_rx)?;
    app.set_ai_channel(ai_tx);

    // Setup terminal
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Enter raw mode
    crossterm::terminal::enable_raw_mode()?;
    let result = Ok(());

    // Event loop
    loop {
        // Draw
        terminal.draw(|frame| {
            ui::render_ui(frame, &app.state);
        })?;

        // Poll for events with timeout
        tokio::select! {
            biased;

            // Check for AI results
            event = app.ai_rx.recv() => {
                match event {
                    Some(AppEvent::AiResult(result)) => {
                        if let Err(e) = app.handle_ai_result(result) {
                            tracing::error!("Failed to handle AI result: {e}");
                        }
                    }
                    None => break,
                }
            }

            // Check for keyboard input
            _ = tokio::task::spawn_blocking(|| {
                event::poll(Duration::from_millis(100))
            }) => {
                if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Char('y') | KeyCode::Char('Y') => {
                                    if let Err(e) = app.accept_idea() {
                                        tracing::error!("{e}");
                                    }
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') => {
                                    if let Err(e) = app.reject_idea() {
                                        tracing::error!("{e}");
                                    }
                                }
                                KeyCode::Char('r') | KeyCode::Char('R') => {
                                    if let Err(e) = app.retry_ai() {
                                        tracing::error!("{e}");
                                    }
                                }
                                KeyCode::Tab => {
                                    app.state.focus = match app.state.focus {
                                        FocusArea::ThoughtList => FocusArea::IdeaPanel,
                                        FocusArea::IdeaPanel => FocusArea::Input,
                                        FocusArea::Input => FocusArea::ThoughtList,
                                    };
                                }
                                KeyCode::Up => {
                                    if app.state.focus == FocusArea::ThoughtList {
                                        app.state.scroll_offset = app.state.scroll_offset.saturating_sub(1);
                                    }
                                }
                                KeyCode::Down => {
                                    if app.state.focus == FocusArea::ThoughtList {
                                        app.state.scroll_offset = app.state.scroll_offset.saturating_add(1);
                                    }
                                }
                                KeyCode::Enter => {
                                    if app.state.focus == FocusArea::Input {
                                        let input = app.state.input.clone();
                                        if input == ":quit" || input == ":q" {
                                            break;
                                        }
                                        if let Err(e) = app.submit_thought(&input) {
                                            tracing::error!("{e}");
                                        }
                                    }
                                }
                                KeyCode::Backspace => {
                                    if app.state.focus == FocusArea::Input && !app.state.input.is_empty() {
                                        let pos = app.cursor_position;
                                        if pos > 0 {
                                            app.state.input.remove(pos - 1);
                                            app.cursor_position = pos - 1;
                                        }
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if app.state.focus == FocusArea::Input {
                                        app.state.input.push(c);
                                        app.cursor_position = app.state.input.len();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    // Cleanup
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    terminal.clear()?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use thinkcloud::db::Database;
    use tokio::sync::mpsc;

    fn setup_test_app() -> App {
        let db = Database::open_in_memory().unwrap();
        let config = Config::default();
        let (_, rx) = mpsc::channel(32);
        App::new(db, config, rx).unwrap()
    }

    #[test]
    fn test_app_creates_default_session() {
        let app = setup_test_app();
        assert!(app.state.current_session.is_some());
        assert_eq!(app.state.sessions.len(), 1);
    }

    #[test]
    fn test_submit_thought() {
        let mut app = setup_test_app();
        app.submit_thought("test thought").unwrap();
        assert_eq!(app.state.thoughts.len(), 1);
        assert_eq!(app.state.thoughts[0].content, "test thought");
        assert!(app.state.input.is_empty());
    }

    #[test]
    fn test_submit_empty_thought() {
        let mut app = setup_test_app();
        app.submit_thought("  ").unwrap();
        assert!(app.state.thoughts.is_empty());
    }

    #[test]
    fn test_accept_idea() {
        let mut app = setup_test_app();
        let session_id = app.state.current_session.as_ref().unwrap().id;

        let idea = app.db.create_idea(session_id, "test idea").unwrap();
        app.state.current_idea = Some(idea);

        app.accept_idea().unwrap();
        assert!(app.state.current_idea.is_none());

        let ideas = app.db.get_accepted_ideas(session_id).unwrap();
        assert_eq!(ideas.len(), 1);
        assert_eq!(ideas[0].status, IdeaStatus::Accepted);
    }

    #[test]
    fn test_reject_idea() {
        let mut app = setup_test_app();
        let session_id = app.state.current_session.as_ref().unwrap().id;

        let idea = app.db.create_idea(session_id, "test idea").unwrap();
        app.state.current_idea = Some(idea);

        app.reject_idea().unwrap();
        assert!(app.state.current_idea.is_none());
    }

    #[test]
    fn test_switch_session() {
        let mut app = setup_test_app();
        let s1 = app.state.current_session.as_ref().unwrap().clone();

        let s2 = app.db.create_session(Some("session 2")).unwrap();
        app.db.create_thought(s2.id, None, "thought in s2").unwrap();

        app.switch_session(s2.id).unwrap();
        assert_eq!(app.state.current_session.as_ref().unwrap().id, s2.id);
        assert_eq!(app.state.thoughts.len(), 1);
        assert_eq!(app.state.thoughts[0].content, "thought in s2");

        app.switch_session(s1.id).unwrap();
        assert_eq!(app.state.current_session.as_ref().unwrap().id, s1.id);
        assert!(app.state.thoughts.is_empty());
    }

    #[test]
    fn test_load_material() {
        let mut app = setup_test_app();
        app.load_material("test.txt").unwrap();
        assert_eq!(app.state.materials.len(), 1);
        assert_eq!(app.state.materials[0].path.as_deref(), Some("test.txt"));

        let session_id = app.state.current_session.as_ref().unwrap().id;
        let mats = app.db.get_session_materials(session_id).unwrap();
        assert_eq!(mats.len(), 1);
    }

    #[test]
    fn test_retry_ai_clears_error_state() {
        let mut app = setup_test_app();
        app.state.status = AppStatus::Error("network error".into());
        app.submit_thought("test for retry").unwrap();

        app.retry_ai().unwrap();
        assert_eq!(app.state.status, AppStatus::Processing);
    }

    #[test]
    fn test_handle_command_material() {
        let mut app = setup_test_app();
        app.handle_command(":material test.txt").unwrap();
        assert_eq!(app.state.materials.len(), 1);
        assert!(app.state.input.is_empty());
    }

    #[test]
    fn test_handle_command_session_new() {
        let mut app = setup_test_app();
        let original_len = app.state.sessions.len();
        app.handle_command(":session new").unwrap();
        assert_eq!(app.state.sessions.len(), original_len + 1);
    }

    #[test]
    fn test_serial_queue_blocks_when_processing() {
        let mut app = setup_test_app();
        app.state.status = AppStatus::Processing;

        app.db.create_thought(
            app.state.current_session.as_ref().unwrap().id,
            None,
            "test",
        ).unwrap();
        app.trigger_ai().unwrap();
        assert_eq!(app.state.status, AppStatus::Processing);
    }

    #[test]
    fn test_serial_queue_drain() {
        let mut app = setup_test_app();
        let session_id = app.state.current_session.as_ref().unwrap().id;

        app.submit_thought("thought 1").unwrap();
        assert_eq!(app.state.status, AppStatus::Processing);

        app.handle_ai_result(Ok("AI result".into())).unwrap();
        assert_eq!(app.state.status, AppStatus::Normal);
        assert!(app.state.current_idea.is_some());
        assert_eq!(app.state.current_idea.as_ref().unwrap().content, "AI result");

        let ideas = app.db.get_recent_ideas(session_id, 10).unwrap();
        assert_eq!(ideas.len(), 1);
        assert_eq!(ideas[0].content, "AI result");
        assert_eq!(ideas[0].status, IdeaStatus::Pending);
    }

    #[test]
    fn test_ai_error_sets_failed_status() {
        let mut app = setup_test_app();

        app.submit_thought("will fail").unwrap();

        app.handle_ai_result(Err("API error".into())).unwrap();
        assert_eq!(app.state.status, AppStatus::Error("API error".into()));

        let has_failed = app.state.thoughts.iter().any(|t| t.status == ThoughtStatus::Failed);
        assert!(has_failed);
    }

    #[test]
    fn test_export_json() {
        let app = setup_test_app();
        app.export_json().unwrap();
    }
}
