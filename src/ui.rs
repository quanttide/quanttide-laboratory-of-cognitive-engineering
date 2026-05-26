use ratatui::{
    prelude::*,
    widgets::{
        Block, BorderType, Borders, List, ListItem, Paragraph, Wrap,
    },
};
use std::vec;

use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusArea {
    ThoughtList,
    IdeaPanel,
    TemplateBar,
    Input,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppStatus {
    Normal,
    Processing,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub thoughts: Vec<Thought>,
    pub current_idea: Option<Idea>,
    pub materials: Vec<Material>,
    pub input: String,
    pub focus: FocusArea,
    pub status: AppStatus,
    pub scroll_offset: usize,
    pub sessions: Vec<Session>,
    pub current_session: Option<Session>,
    pub templates: Vec<String>,
    pub template_focus_index: usize,
}

impl UiState {
    pub fn empty() -> Self {
        Self {
            thoughts: vec![],
            current_idea: None,
            materials: vec![],
            input: String::new(),
            focus: FocusArea::Input,
            status: AppStatus::Normal,
            scroll_offset: 0,
            sessions: vec![],
            current_session: None,
            templates: vec![],
            template_focus_index: 0,
        }
    }
}

pub fn render_ui(frame: &mut Frame, state: &UiState) {
    let area = frame.area();

    let has_templates = !state.templates.is_empty();
    let template_rows = if has_templates { 1 } else { 0 };

    // Layout: status bar, middle (thoughts|idea), template bar, input, help bar
    let main_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(template_rows * 3),
        Constraint::Length(3),
        Constraint::Length(1),
    ]);
    let [status_bar, middle, template_area, input_area, help_bar] = main_layout.areas(area);

    // Middle: split into left (thoughts) and right (idea)
    let middle_layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [left_panel, right_panel] = middle_layout.areas(middle);

    render_status_bar(frame, status_bar, state);
    render_thought_panel(frame, left_panel, state);
    render_idea_panel(frame, right_panel, state);

    if has_templates {
        render_template_bar(frame, template_area, state);
    }

    render_input(frame, input_area, state);
    render_help_bar(frame, help_bar, state);
}

fn render_status_bar(frame: &mut Frame, area: Rect, state: &UiState) {
    let material_info = if state.materials.is_empty() {
        "材料: (无)".to_string()
    } else {
        let names: Vec<&str> = state
            .materials
            .iter()
            .filter_map(|m| m.path.as_deref())
            .collect();
        format!("材料: {}", names.join(", "))
    };

    let session_info = match &state.current_session {
        Some(s) => format!("Session: {}", s.title.as_deref().unwrap_or("未命名")),
        None => "Session: (无)".to_string(),
    };

    let status_info = match &state.status {
        AppStatus::Processing => " [AI 处理中...]".to_string(),
        AppStatus::Error(e) => format!(" [错误: {e}]"),
        AppStatus::Normal => String::new(),
    };

    let title = format!(
        "思考云  │  {}  │  {}{}",
        material_info, session_info, status_info
    );

    let paragraph = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(paragraph, area);
}

fn render_thought_panel(frame: &mut Frame, area: Rect, state: &UiState) {
    let is_focused = state.focus == FocusArea::ThoughtList;

    let items: Vec<ListItem> = if state.thoughts.is_empty() {
        // P0: Empty state for thoughts
        vec![ListItem::new(
            Span::styled(
                "还没有念头\n在底部输入框输入你的第一个念头",
                Style::default().fg(Color::DarkGray),
            ),
        )]
    } else {
        state
            .thoughts
            .iter()
            .rev()
            .map(|t| {
                let prefix = match t.status {
                    ThoughtStatus::Pending => "○",
                    ThoughtStatus::Processing => "⟳",
                    ThoughtStatus::Completed => "✓",
                    ThoughtStatus::Failed => "✗",
                };
                let style = match t.status {
                    ThoughtStatus::Failed => Style::default().fg(Color::Red),
                    _ => Style::default(),
                };
                ListItem::new(format!("{prefix} {}", t.content)).style(style)
            })
            .collect()
    };

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" 念头流 ")
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(border_style);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, area);
}

fn render_idea_panel(frame: &mut Frame, area: Rect, state: &UiState) {
    let is_focused = state.focus == FocusArea::IdeaPanel;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" AI 想法 ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let content = match &state.status {
        AppStatus::Error(e) => {
            // P0: Error state with retry hint
            format!(
                "⚠ AI 调用失败\n\n{e}\n\n[r] 重试"
            )
        }
        AppStatus::Processing => {
            "⟳ AI 正在思考...\n\n请稍候".to_string()
        }
        AppStatus::Normal => {
            match &state.current_idea {
                None => {
                    // P0: Empty state for ideas
                    "AI 想法将在这里显示\n\n输入念头后，AI 会自动生成想法".to_string()
                }
                Some(idea) => {
                    format!(
                        "{}\n\n[y] 接受  [n] 拒绝",
                        idea.content
                    )
                }
            }
        }
    };

    let style = match &state.status {
        AppStatus::Error(_) => Style::default().fg(Color::Red),
        AppStatus::Processing => Style::default().fg(Color::Yellow),
        AppStatus::Normal => Style::default(),
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(style)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_template_bar(frame: &mut Frame, area: Rect, state: &UiState) {
    let is_focused = state.focus == FocusArea::TemplateBar;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" 提示词模板 ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = state
        .templates
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let prefix = if is_focused && i == state.template_focus_index {
                "▸ "
            } else {
                "  "
            };
            ListItem::new(format!("{prefix}{t}"))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, area);
}

fn render_input(frame: &mut Frame, area: Rect, state: &UiState) {
    let is_focused = state.focus == FocusArea::Input;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let input_text = if state.input.is_empty() && !is_focused {
        "输入新念头...".to_string()
    } else {
        state.input.clone()
    };

    let input_style = if state.input.is_empty() && !is_focused {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };

    let paragraph = Paragraph::new(format!("> {}", input_text))
        .block(block)
        .style(input_style);

    frame.render_widget(paragraph, area);
}

fn render_help_bar(frame: &mut Frame, area: Rect, state: &UiState) {
    let has_templates = !state.templates.is_empty();
    let help_text = match state.status {
        AppStatus::Error(_) => {
            " Tab切换焦点  ↑↓模板  Enter选用  r重试  q退出"
        }
        _ => {
            if has_templates {
                " Tab切换焦点  ↑↓模板  Enter选用  y接受  n拒绝  Enter发送  q退出"
            } else {
                " Tab切换焦点  ↑↓滚动  y接受  n拒绝  Enter发送  /命令  q退出"
            }
        }
    };

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(paragraph, area);
}

pub fn render_to_string(state: &UiState, width: u16, height: u16) -> String {
    let backend = ratatui::backend::TestBackend::new(width, height);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            render_ui(frame, state);
        })
        .unwrap();
    let buffer = terminal.backend().buffer();
    let mut result = String::new();
    for y in 0..height {
        let mut line = String::new();
        let mut x = 0;
        while x < width {
            let cell = &buffer[(x, y)];
            let symbol = cell.symbol();
            if symbol.is_empty() {
                x += 1;
                continue;
            }
            if symbol == " " {
                line.push(' ');
            } else {
                line.push_str(symbol);
            }
            x += 1;
        }
        let trimmed = line.trim_end().to_string();
        result.push_str(&trimmed);
        if y < height - 1 {
            result.push('\n');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Normalize whitespace for CJK testing: remove spaces to handle
    /// ratatui TestBackend double-width CJK rendering differences
    fn compact(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    fn sample_state() -> UiState {
        UiState {
            thoughts: vec![
                Thought {
                    id: 1,
                    session_id: 1,
                    material_id: None,
                    content: "第一个念头".into(),
                    status: ThoughtStatus::Pending,
                    sort_order: 1,
                    created_at: 1000,
                },
                Thought {
                    id: 2,
                    session_id: 1,
                    material_id: None,
                    content: "第二个念头".into(),
                    status: ThoughtStatus::Completed,
                    sort_order: 2,
                    created_at: 1001,
                },
            ],
            current_idea: Some(Idea {
                id: 1,
                session_id: 1,
                content: "AI生成的结论".into(),
                status: IdeaStatus::Pending,
                sort_order: 1,
                created_at: 1002,
            }),
            materials: vec![Material {
                id: 1,
                path: Some("bug.txt".into()),
                content_snippet: Some("bug desc".into()),
                created_at: 900,
            }],
            input: "test input".to_string(),
            focus: FocusArea::Input,
            status: AppStatus::Normal,
            scroll_offset: 0,
            sessions: vec![Session {
                id: 1,
                title: Some("test".into()),
                created_at: 100,
                updated_at: 1002,
                ai_pending: false,
            }],
            current_session: Some(Session {
                id: 1,
                title: Some("test".into()),
                created_at: 100,
                updated_at: 1002,
                ai_pending: false,
            }),
        }
    }

    fn test_render_inner(state: &UiState) -> String {
        render_to_string(state, 80, 24)
    }

    #[test]
    fn test_render_empty_state() {
        let state = UiState::empty();
        let output = test_render_inner(&state);
        let compacted = compact(&output);

        assert!(compacted.contains("还没有念头"), "Empty state should show thought placeholder. Output:\n{output}");
        assert!(compacted.contains("AI想法将在这里显示"), "Empty state should show idea placeholder. Output:\n{output}");
        assert!(compacted.contains("材料"), "Should show materials section. Output:\n{output}");
    }

    #[test]
    fn test_render_with_data() {
        let state = sample_state();
        let output = test_render_inner(&state);
        let compacted = compact(&output);

        assert!(compacted.contains("第一个念头"), "Should show thought. Output:\n{output}");
        assert!(compacted.contains("第二个念头"), "Should show thought. Output:\n{output}");
        assert!(compacted.contains("bug.txt"), "Should show material. Output:\n{output}");
    }

    #[test]
    fn test_render_processing_state() {
        let mut state = sample_state();
        state.status = AppStatus::Processing;
        let output = test_render_inner(&state);
        let compacted = compact(&output);

        assert!(compacted.contains("AI正在思考"), "Should show processing message. Output:\n{output}");
    }

    #[test]
    fn test_render_error_state() {
        let mut state = sample_state();
        state.status = AppStatus::Error("网络连接失败".into());
        let output = test_render_inner(&state);
        let compacted = compact(&output);

        assert!(compacted.contains("网络连接失败"), "Should show error message. Output:\n{output}");
        assert!(compacted.contains("r重试"), "Should show retry hint. Output:\n{output}");
    }

    #[test]
    fn test_render_failed_thought() {
        let mut state = sample_state();
        state.thoughts[0].status = ThoughtStatus::Failed;
        let output = test_render_inner(&state);
        let compacted = compact(&output);

        assert!(compacted.contains('✗'), "Failed thought should show ✗");
    }

    #[test]
    fn test_focus_cycle() {
        let mut state = sample_state();
        // Start with Input focus
        assert_eq!(state.focus, FocusArea::Input);

        // Cycle: Input -> ThoughtList
        state.focus = match state.focus {
            FocusArea::ThoughtList => FocusArea::IdeaPanel,
            FocusArea::IdeaPanel => FocusArea::Input,
            FocusArea::Input => FocusArea::ThoughtList,
        };
        assert_eq!(state.focus, FocusArea::ThoughtList);

        // Cycle: ThoughtList -> IdeaPanel
        state.focus = match state.focus {
            FocusArea::ThoughtList => FocusArea::IdeaPanel,
            FocusArea::IdeaPanel => FocusArea::Input,
            FocusArea::Input => FocusArea::ThoughtList,
        };
        assert_eq!(state.focus, FocusArea::IdeaPanel);

        // Cycle: IdeaPanel -> Input
        state.focus = match state.focus {
            FocusArea::ThoughtList => FocusArea::IdeaPanel,
            FocusArea::IdeaPanel => FocusArea::Input,
            FocusArea::Input => FocusArea::ThoughtList,
        };
        assert_eq!(state.focus, FocusArea::Input);
    }

    #[test]
    fn test_render_multiline_empty_idea() {
        let state = UiState::empty();
        let output = test_render_inner(&state);
        let compacted = compact(&output);

        assert!(compacted.contains("AI想法将在这里显示"));
    }
}
