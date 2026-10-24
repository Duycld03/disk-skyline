use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use std::fs;
use std::path::PathBuf;

use crate::model::{format_bytes, BuildingNode};

pub const LARGE_SIZE_THRESHOLD: u64 = 1024 * 1024 * 1024; // 1 GB

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemolishAction {
    Pending,
    Confirmed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct DemolishState {
    pub target_path: PathBuf,
    pub target_name: String,
    pub target_size: u64,
    pub is_restricted: bool,
    pub is_dir: bool,
    pub input_buffer: String,
    pub requires_explicit_yes: bool,
    pub is_active: bool,
    pub error_message: Option<String>,
}

impl DemolishState {
    pub fn new(node: &BuildingNode) -> Self {
        let is_restricted = node.is_hidden;
        let requires_explicit_yes = is_restricted || node.size >= LARGE_SIZE_THRESHOLD;

        Self {
            target_path: node.path.clone(),
            target_name: node.name.clone(),
            target_size: node.size,
            is_restricted,
            is_dir: node.is_dir,
            input_buffer: String::new(),
            requires_explicit_yes,
            is_active: true,
            error_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> DemolishAction {
        match key.code {
            KeyCode::Esc => {
                self.is_active = false;
                DemolishAction::Cancelled
            }
            KeyCode::Char('n') | KeyCode::Char('N') if !self.requires_explicit_yes => {
                self.is_active = false;
                DemolishAction::Cancelled
            }
            KeyCode::Char('y') | KeyCode::Char('Y') if !self.requires_explicit_yes => {
                self.is_active = false;
                DemolishAction::Confirmed
            }
            KeyCode::Char(c) if self.requires_explicit_yes => {
                if self.input_buffer.len() < 10 {
                    self.input_buffer.push(c);
                }
                DemolishAction::Pending
            }
            KeyCode::Backspace if self.requires_explicit_yes => {
                self.input_buffer.pop();
                DemolishAction::Pending
            }
            KeyCode::Enter if self.requires_explicit_yes => {
                if self.input_buffer.trim().eq_ignore_ascii_case("YES") {
                    self.is_active = false;
                    DemolishAction::Confirmed
                } else {
                    self.error_message = Some("Cần gõ chính xác chữ 'YES' để xác nhận!".into());
                    DemolishAction::Pending
                }
            }
            _ => DemolishAction::Pending,
        }
    }
}

/// Executes filesystem deletion and mutates in-memory tree directly without full rescan
pub fn execute_demolition(
    state: &DemolishState,
    parent_node: &mut BuildingNode,
    selected_index: usize,
) -> Result<u64, std::io::Error> {
    // 1. Filesystem operation or PurgeAction execution
    let custom_purge = if selected_index < parent_node.children.len() {
        parent_node.children[selected_index].purge_action.clone()
    } else {
        None
    };

    if let Some(action) = custom_purge {
        action.execute().map_err(std::io::Error::other)?;
    } else if state.target_path.exists() {
        if state.is_dir {
            fs::remove_dir_all(&state.target_path)?;
        } else {
            fs::remove_file(&state.target_path)?;
        }
    }
    // 2. Direct In-Memory mutation
    let freed_bytes = if selected_index < parent_node.children.len() {
        let removed = parent_node.children.remove(selected_index);
        parent_node.size = parent_node.size.saturating_sub(removed.size);
        removed.size
    } else {
        0
    };

    Ok(freed_bytes)
}

/// Executes sequential bulldozer purge across all cleanable targets in the Scrapyard Avenue
pub fn execute_bulldozer_purge(parent_node: &mut BuildingNode) -> Result<u64, std::io::Error> {
    let mut total_freed = 0u64;
    let mut i = 0;
    while i < parent_node.children.len() {
        if let Some(action) = parent_node.children[i].purge_action.clone() {
            match action.execute() {
                Ok(()) => {
                    let removed = parent_node.children.remove(i);
                    total_freed = total_freed.saturating_add(removed.size);
                }
                Err(_) => {
                    // Retain target in avenue if purge failed (e.g. sudo required or daemon down)
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    parent_node.size = parent_node.size.saturating_sub(total_freed);
    Ok(total_freed)
}

/// Renders modal overlay for demolition confirmation
pub fn render_demolish_modal(
    state: &DemolishState,
    language: crate::i18n::Language,
    frame: &mut Frame,
) {
    let screen = frame.area();
    let modal_width = 64.min(screen.width.saturating_sub(4));
    let modal_height = 12.min(screen.height.saturating_sub(2));

    let x = (screen.width.saturating_sub(modal_width)) / 2;
    let y = (screen.height.saturating_sub(modal_height)) / 2;
    let area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, area);

    let border_color = if state.requires_explicit_yes {
        Color::Red
    } else {
        Color::Yellow
    };

    let title = match (language, state.requires_explicit_yes) {
        (crate::i18n::Language::Vi, true) => " [!] LỆNH PHÁ DỠ ĐẶC BIỆT (NGUY HIỂM) ",
        (crate::i18n::Language::Vi, false) => " [!] XÁC NHẬN PHÁ DỠ CÔNG TRÌNH ",
        (crate::i18n::Language::En, true) => " [!] HIGH-RISK DEMOLITION ORDER [!] ",
        (crate::i18n::Language::En, false) => " [!] CONFIRM BUILDING DEMOLITION ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .title(title);

    let target_label = match language {
        crate::i18n::Language::Vi => "Mục tiêu: ",
        crate::i18n::Language::En => "Target: ",
    };
    let reclaimed_label = match language {
        crate::i18n::Language::Vi => "Giải phóng mặt bằng: ",
        crate::i18n::Language::En => "Reclaimed space: ",
    };

    let mut text_lines = vec![
        Line::from(vec![
            Span::styled(target_label, Style::default().fg(Color::DarkGray)),
            Span::styled(
                state.target_path.to_string_lossy(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(reclaimed_label, Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_bytes(state.target_size),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    if state.requires_explicit_yes {
        let warn_msg = match language {
            crate::i18n::Language::Vi => {
                "CẢNH BÁO: Đây là khu vực cấm hoặc công trình quy mô lớn (>= 1 GB)!"
            }
            crate::i18n::Language::En => {
                "WARNING: Target is a restricted bunker or large building (>= 1 GB)!"
            }
        };
        text_lines.push(Line::from(Span::styled(
            warn_msg,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        let prompt_prefix = match language {
            crate::i18n::Language::Vi => "Gõ chữ ",
            crate::i18n::Language::En => "Type ",
        };
        let prompt_suffix = match language {
            crate::i18n::Language::Vi => " rồi bấm Enter để xác nhận: [",
            crate::i18n::Language::En => " and press Enter to confirm: [",
        };
        text_lines.push(Line::from(vec![
            Span::styled(prompt_prefix, Style::default().fg(Color::White)),
            Span::styled(
                "YES",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(prompt_suffix, Style::default().fg(Color::White)),
            Span::styled(
                &state.input_buffer,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("]", Style::default().fg(Color::White)),
        ]));
    } else {
        let warn_msg = match language {
            crate::i18n::Language::Vi => "Hành động này sẽ XÓA VĨNH VIỄN file/thư mục trên đĩa!",
            crate::i18n::Language::En => {
                "This action will PERMANENTLY DELETE file/directory from disk!"
            }
        };
        text_lines.push(Line::from(Span::styled(
            warn_msg,
            Style::default().fg(Color::LightRed),
        )));
        let press_word = match language {
            crate::i18n::Language::Vi => "Nhấn ",
            crate::i18n::Language::En => "Press ",
        };
        let to_demolish = match language {
            crate::i18n::Language::Vi => " để phá dỡ, hoặc ",
            crate::i18n::Language::En => " to demolish, or ",
        };
        let to_cancel = match language {
            crate::i18n::Language::Vi => " để hủy bỏ.",
            crate::i18n::Language::En => " to cancel.",
        };
        text_lines.push(Line::from(vec![
            Span::styled(press_word, Style::default().fg(Color::White)),
            Span::styled(
                "[y]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(to_demolish, Style::default().fg(Color::White)),
            Span::styled(
                "[Esc/n]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(to_cancel, Style::default().fg(Color::White)),
        ]));
    }

    if let Some(err) = &state.error_message {
        text_lines.push(Line::from(Span::styled(
            format!("* {}", err),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    let paragraph = Paragraph::new(text_lines)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demolish_two_tier_rules() {
        let normal_file = BuildingNode::new(
            "log.txt".into(),
            PathBuf::from("log.txt"),
            false,
            false,
            5000,
            vec![],
        );
        let state1 = DemolishState::new(&normal_file);
        assert!(!state1.requires_explicit_yes);

        let restricted_dotfile = BuildingNode::new(
            ".env".into(),
            PathBuf::from(".env"),
            false,
            true,
            200,
            vec![],
        );
        let state2 = DemolishState::new(&restricted_dotfile);
        assert!(state2.requires_explicit_yes);

        let huge_file = BuildingNode::new(
            "data.vdi".into(),
            PathBuf::from("data.vdi"),
            false,
            false,
            2 * 1024 * 1024 * 1024,
            vec![],
        );
        let state3 = DemolishState::new(&huge_file);
        assert!(state3.requires_explicit_yes);
    }

    #[test]
    fn test_demolish_in_memory_tree_pruning() {
        let mut parent = BuildingNode::new(
            "root".into(),
            PathBuf::from("/root"),
            true,
            false,
            1500,
            vec![
                BuildingNode::new(
                    "f1.txt".into(),
                    PathBuf::from("/root/f1.txt"),
                    false,
                    false,
                    500,
                    vec![],
                ),
                BuildingNode::new(
                    "f2.txt".into(),
                    PathBuf::from("/root/f2.txt"),
                    false,
                    false,
                    1000,
                    vec![],
                ),
            ],
        );

        let state = DemolishState::new(&parent.children[0]);
        // Target does not exist on disk in test, execute_demolition should still prune memory
        let freed = execute_demolition(&state, &mut parent, 0).unwrap();

        assert_eq!(freed, 500);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].name, "f2.txt");
        assert_eq!(parent.size, 1000);
    }

    #[test]
    fn test_demolish_with_purge_action() {
        let temp_dir = std::env::temp_dir().join("test_purge_demolish_node");
        let _ = std::fs::create_dir_all(&temp_dir);
        let sample = temp_dir.join("sample.log");
        let _ = std::fs::write(&sample, b"junk logs");
        assert!(sample.exists());

        let mut child = BuildingNode::new(
            "Junk Target".into(),
            temp_dir.clone(),
            true,
            false,
            5000,
            vec![],
        );
        child.purge_action = Some(crate::cleaner::PurgeAction::DirectoryDelete(
            temp_dir.clone(),
        ));

        let mut parent = BuildingNode::new(
            "Avenue".into(),
            PathBuf::from("/avenue"),
            true,
            false,
            5000,
            vec![child],
        );

        let state = DemolishState::new(&parent.children[0]);
        let res = execute_demolition(&state, &mut parent, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 5000);
        assert!(parent.children.is_empty());
        assert_eq!(parent.size, 0);
        assert!(!sample.exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_execute_bulldozer_purge() {
        let temp_dir = std::env::temp_dir().join("test_bulldozer_purge_dir");
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut child1 =
            BuildingNode::new("Scrap1".into(), temp_dir.clone(), true, false, 3000, vec![]);
        child1.purge_action = Some(crate::cleaner::PurgeAction::DirectoryDelete(
            temp_dir.clone(),
        ));

        let mut parent = BuildingNode::new(
            "Scrapyard Avenue".into(),
            PathBuf::from("//scrapyard"),
            true,
            false,
            3000,
            vec![child1],
        );

        let freed = execute_bulldozer_purge(&mut parent).unwrap();
        assert_eq!(freed, 3000);
        assert!(parent.children.is_empty());
        assert_eq!(parent.size, 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
