use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::demolish::{execute_demolition, DemolishAction, DemolishState};
use crate::model::{BuildingNode, NavigationState};
use crate::scanner::{scan_directory, ScannerOptions};
use crate::skyline::SkylinePlacement;

pub struct App {
    pub root_node: BuildingNode,
    pub nav: NavigationState,
    pub scanner_options: ScannerOptions,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub should_quit: bool,
    pub demolish_state: Option<DemolishState>,
    pub language: crate::i18n::Language,
}

impl App {
    pub fn new(root_node: BuildingNode, scanner_options: ScannerOptions) -> Self {
        Self {
            root_node,
            nav: NavigationState::new(),
            scanner_options,
            status_message: None,
            should_quit: false,
            demolish_state: None,
            language: crate::i18n::Language::default(),
        }
    }

    pub fn with_language(mut self, language: crate::i18n::Language) -> Self {
        self.language = language;
        self
    }

    /// Retrieves a reference to the active directory node based on navigation breadcrumbs
    pub fn current_node(&self) -> &BuildingNode {
        let mut curr = &self.root_node;
        for &idx in &self.nav.current_path_indices {
            if idx < curr.children.len() {
                curr = &curr.children[idx];
            } else {
                break;
            }
        }
        curr
    }

    /// Retrieves a mutable reference to the active directory node
    pub fn current_node_mut(&mut self) -> &mut BuildingNode {
        let mut curr = &mut self.root_node;
        for &idx in &self.nav.current_path_indices {
            if idx < curr.children.len() {
                curr = &mut curr.children[idx];
            } else {
                break;
            }
        }
        curr
    }

    pub fn current_selected_child(&self) -> Option<&BuildingNode> {
        let node = self.current_node();
        node.children.get(self.nav.selected_index)
    }

    pub fn breadcrumbs(&self) -> Vec<&str> {
        let mut trail = vec![self.root_node.name.as_str()];
        let mut curr = &self.root_node;
        for &idx in &self.nav.current_path_indices {
            if idx < curr.children.len() {
                curr = &curr.children[idx];
                trail.push(curr.name.as_str());
            } else {
                break;
            }
        }
        trail
    }

    pub fn move_left(&mut self) {
        self.nav.move_left();
    }

    pub fn move_right(&mut self) {
        let count = self.current_node().children.len();
        self.nav.move_right(count);
    }

    pub fn drill_down(&mut self) -> bool {
        let child_info = {
            let current_node = self.current_node();
            current_node
                .children
                .get(self.nav.selected_index)
                .map(|child| (child.is_dir, child.children.len(), child.name.clone()))
        };

        if let Some((is_dir, child_count, child_name)) = child_info {
            if is_dir && child_count > 0 {
                let success = self.nav.drill_down(child_count);
                if success {
                    self.status_message =
                        Some((self.language.drill_down_entered(&child_name), false));
                }
                return success;
            } else if is_dir {
                self.status_message =
                    Some((self.language.drill_down_empty_err().to_string(), false));
            } else {
                self.status_message = Some((self.language.drill_down_file_err(&child_name), false));
            }
        }
        false
    }

    pub fn navigate_up(&mut self) -> bool {
        let success = self.nav.navigate_up();
        if success {
            self.status_message = Some((self.language.navigate_up_success().to_string(), false));
        }
        success
    }

    pub fn toggle_sort(&mut self) {
        self.nav.sort_mode = self.nav.sort_mode.next();
        let mode = self.nav.sort_mode;
        self.current_node_mut().sort_children(mode);
        let mode_label = mode.label_lang(self.language);
        self.status_message = Some((self.language.zoning_switched(mode_label), false));
    }

    pub fn rescan_current(&mut self) -> Result<(), std::io::Error> {
        let target_path = self.current_node().path.clone();
        match scan_directory(&target_path, &self.scanner_options) {
            Ok(new_subtree) => {
                let sort_mode = self.nav.sort_mode;
                let curr = self.current_node_mut();
                curr.size = new_subtree.size;
                curr.children = new_subtree.children;
                curr.sort_children(sort_mode);
                let child_count = curr.children.len();
                if self.nav.selected_index >= child_count {
                    self.nav.selected_index = child_count.saturating_sub(1);
                }
                self.status_message = Some((self.language.rescan_success(child_count), false));
                Ok(())
            }
            Err(e) => {
                self.status_message = Some((format!("Lỗi làm mới: {}", e), true));
                Err(e)
            }
        }
    }

    pub fn update_camera_viewport(&mut self, screen_width: u16, placements: &[SkylinePlacement]) {
        if placements.is_empty() || screen_width == 0 {
            self.nav.viewport_offset_x = 0;
            return;
        }

        if let Some(p) = placements.get(self.nav.selected_index) {
            let left_edge = p.start_col;
            let right_edge = p.start_col + p.width as usize;
            let view_width = screen_width as usize;

            // If building is to the left of the camera
            if left_edge < self.nav.viewport_offset_x {
                self.nav.viewport_offset_x = left_edge.saturating_sub(4);
            }
            // If building is to the right of the camera
            else if right_edge > self.nav.viewport_offset_x + view_width {
                self.nav.viewport_offset_x = (right_edge + 4).saturating_sub(view_width);
            }
        }
    }
    pub fn handle_key(&mut self, key: KeyEvent) {
        // If demolition modal is active, delegate keys to modal
        if let Some(mut state) = self.demolish_state.take() {
            match state.handle_key(key) {
                DemolishAction::Pending => {
                    self.demolish_state = Some(state);
                }
                DemolishAction::Cancelled => {
                    self.status_message = Some((self.language.demolish_cancelled().into(), false));
                    self.demolish_state = None;
                }
                DemolishAction::Confirmed => {
                    let selected_idx = self.nav.selected_index;
                    let target_name = state.target_name.clone();
                    let (res, remaining_count) = {
                        let curr = self.current_node_mut();
                        let result = execute_demolition(&state, curr, selected_idx);
                        (result, curr.children.len())
                    };

                    match res {
                        Ok(freed) => {
                            if self.nav.selected_index >= remaining_count {
                                self.nav.selected_index = remaining_count.saturating_sub(1);
                            }
                            let msg = if self.language == crate::i18n::Language::Vi {
                                format!(
                                    "Đã phá dỡ thành công '{}', giải phóng {}!",
                                    target_name,
                                    crate::model::format_bytes(freed)
                                )
                            } else {
                                format!(
                                    "Successfully demolished '{}', freed {}!",
                                    target_name,
                                    crate::model::format_bytes(freed)
                                )
                            };
                            self.status_message = Some((msg, false));
                        }
                        Err(e) => {
                            self.status_message = Some((format!("Lỗi khi phá dỡ: {}", e), true));
                        }
                    }
                    self.demolish_state = None;
                }
            }
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.move_left();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.move_right();
            }
            KeyCode::Enter => {
                self.drill_down();
            }
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('u') => {
                self.navigate_up();
            }
            KeyCode::Tab => {
                self.toggle_sort();
            }
            KeyCode::Char('r') | KeyCode::F(5) => {
                let _ = self.rescan_current();
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(child) = self.current_selected_child() {
                    if child.is_system_protected {
                        self.status_message =
                            Some((self.language.protected_rejection_notice().into(), true));
                    } else {
                        self.demolish_state = Some(DemolishState::new(child));
                    }
                }
            }
            KeyCode::Char('L') => {
                self.language = self.language.toggle();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_mock_tree() -> BuildingNode {
        let sub_child1 = BuildingNode::new(
            "file1.rs".into(),
            PathBuf::from("file1.rs"),
            false,
            false,
            100,
            vec![],
        );
        let sub_child2 = BuildingNode::new(
            "file2.rs".into(),
            PathBuf::from("file2.rs"),
            false,
            false,
            200,
            vec![],
        );

        let dir1 = BuildingNode::new(
            "src".into(),
            PathBuf::from("src"),
            true,
            false,
            300,
            vec![sub_child1, sub_child2],
        );
        let dir2 = BuildingNode::new(
            "docs".into(),
            PathBuf::from("docs"),
            true,
            false,
            50,
            vec![],
        );

        BuildingNode::new(
            "root".into(),
            PathBuf::from("/root"),
            true,
            false,
            350,
            vec![dir1, dir2],
        )
    }

    #[test]
    fn test_app_navigation_and_breadcrumbs() {
        let tree = create_mock_tree();
        let mut app = App::new(tree, ScannerOptions::default());

        assert_eq!(app.breadcrumbs(), vec!["root"]);
        assert_eq!(app.current_selected_child().unwrap().name, "src");

        // Move right to docs
        app.move_right();
        assert_eq!(app.nav.selected_index, 1);
        assert_eq!(app.current_selected_child().unwrap().name, "docs");

        // Move left back to src
        app.move_left();
        assert_eq!(app.nav.selected_index, 0);

        // Drill down into src
        assert!(app.drill_down());
        assert_eq!(app.breadcrumbs(), vec!["root", "src"]);
        assert_eq!(app.current_selected_child().unwrap().name, "file1.rs");

        // Navigate up
        assert!(app.navigate_up());
        assert_eq!(app.breadcrumbs(), vec!["root"]);
        assert_eq!(app.current_selected_child().unwrap().name, "src");
    }

    #[test]
    fn test_camera_viewport_update() {
        let tree = create_mock_tree();
        let mut app = App::new(tree, ScannerOptions::default());

        let placements = vec![
            SkylinePlacement {
                node_index: 0,
                start_col: 0,
                width: 10,
                height: 10,
            },
            SkylinePlacement {
                node_index: 1,
                start_col: 100,
                width: 10,
                height: 10,
            },
        ];

        // Initially at index 0, screen width 50
        app.update_camera_viewport(50, &placements);
        assert_eq!(app.nav.viewport_offset_x, 0);

        // Move to index 1 (start_col 100, which is outside 50)
        app.nav.selected_index = 1;
        app.update_camera_viewport(50, &placements);
        assert!(app.nav.viewport_offset_x > 50);
    }

    #[test]
    fn test_system_protected_demolish_rejection() {
        let protected = BuildingNode::new(
            "etc".into(),
            PathBuf::from("/etc"),
            true,
            false,
            500,
            vec![],
        );
        assert!(protected.is_system_protected);

        let root = BuildingNode::new(
            "root".into(),
            PathBuf::from("/"),
            true,
            false,
            500,
            vec![protected],
        );
        let mut app = App::new(root, ScannerOptions::default());

        let key = KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_key(key);

        assert!(
            app.demolish_state.is_none(),
            "Demolish state must remain None for protected target"
        );
        assert!(
            app.status_message.is_some(),
            "Status message must indicate rejection"
        );
        assert!(
            app.status_message.unwrap().1,
            "Rejection status message must be flagged as error"
        );
    }

    #[test]
    fn test_navigation_localized_messages() {
        let file_child = BuildingNode::new(
            "test.txt".into(),
            PathBuf::from("/test.txt"),
            false,
            false,
            100,
            vec![],
        );
        let root = BuildingNode::new(
            "root".into(),
            PathBuf::from("/"),
            true,
            false,
            100,
            vec![file_child],
        );
        let mut app = App::new(root, ScannerOptions::default());

        // Vietnamese mode (default)
        app.language = crate::i18n::Language::Vi;
        let drilled = app.drill_down();
        assert!(!drilled);
        assert!(app.status_message.is_some());
        let (vi_msg, _) = app.status_message.clone().unwrap();
        assert!(vi_msg.contains("không thể đi vào"));

        // Switch to English mode
        app.language = crate::i18n::Language::En;
        let drilled = app.drill_down();
        assert!(!drilled);
        assert!(app.status_message.is_some());
        let (en_msg, _) = app.status_message.clone().unwrap();
        assert!(en_msg.contains("is a single file, cannot enter"));
    }
}
