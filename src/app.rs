use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use crate::demolish::{
    execute_demolition, DemolishAction, DemolishState, SudoAction, SudoModalState,
};
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
    pub sudo_modal_state: Option<SudoModalState>,
    pub language: crate::i18n::Language,
    pub search_query: Option<String>,
    pub pre_search_index: Option<usize>,
    pub saved_root_node: Option<BuildingNode>,
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
            sudo_modal_state: None,
            language: crate::i18n::Language::default(),
            search_query: None,
            pre_search_index: None,
            saved_root_node: None,
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
        if self.nav.current_path_indices.is_empty() {
            if self.saved_root_node.is_some() {
                self.toggle_scrapyard();
                return true;
            }
            return false;
        }
        let success = self.nav.navigate_up();
        if success {
            self.status_message = Some((self.language.navigate_up_success().to_string(), false));
        }
        success
    }

    pub fn is_in_scrapyard(&self) -> bool {
        self.saved_root_node.is_some() || self.root_node.name.contains("SCRAPYARD")
    }

    pub fn toggle_scrapyard(&mut self) -> bool {
        if let Some(prev_root) = self.saved_root_node.take() {
            self.root_node = prev_root;
            self.nav = NavigationState::new();
            self.status_message = Some((self.language.scrapyard_exited().to_string(), false));
            false
        } else if self.root_node.name.contains("SCRAPYARD") {
            let options = self.scanner_options;
            let target_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if let Ok(fs_root) = scan_directory(&target_path, &options) {
                self.root_node = fs_root;
            }
            self.nav = NavigationState::new();
            self.status_message = Some((self.language.scrapyard_exited().to_string(), false));
            false
        } else {
            let targets = crate::cleaner::survey_all_system_junk();
            let prev = std::mem::replace(
                &mut self.root_node,
                BuildingNode::build_scrapyard_avenue(&targets),
            );
            self.saved_root_node = Some(prev);
            self.nav = NavigationState::new();
            self.status_message = Some((self.language.scrapyard_entered().to_string(), false));
            true
        }
    }

    pub fn execute_bulldozer(&mut self, sudo_password: Option<&str>) -> Result<u64, std::io::Error> {
        let freed = crate::demolish::execute_bulldozer_purge(&mut self.root_node, sudo_password)?;
        self.nav.selected_index = 0;
        let bytes_str = crate::model::format_bytes(freed);
        self.status_message = Some((self.language.bulldozer_success(&bytes_str), false));
        Ok(freed)
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
    pub fn open_current_selected(&mut self) -> bool {
        let child_info = {
            let current_node = self.current_node();
            current_node
                .children
                .get(self.nav.selected_index)
                .map(|child| (child.name.clone(), child.path.clone()))
        };

        if let Some((name, path)) = child_info {
            match launch_default_app(&path) {
                Ok(_) => {
                    self.status_message = Some((self.language.open_success(&name), false));
                    true
                }
                Err(_) => {
                    self.status_message = Some((self.language.open_err(&name), true));
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn start_search(&mut self) {
        self.pre_search_index = Some(self.nav.selected_index);
        self.search_query = Some(String::new());
    }

    pub fn update_search(&mut self, ch: char) {
        if let Some(ref mut query) = self.search_query {
            if query.len() < 48 {
                query.push(ch);
            }
        }
        self.jump_to_search_match();
    }

    pub fn backspace_search(&mut self) {
        if let Some(ref mut query) = self.search_query {
            query.pop();
        }
        self.jump_to_search_match();
    }

    pub fn cancel_search(&mut self) {
        self.search_query = None;
        if let Some(idx) = self.pre_search_index.take() {
            self.nav.selected_index = idx;
        }
    }

    pub fn confirm_search(&mut self) {
        self.search_query = None;
        self.pre_search_index = None;
    }

    pub fn jump_to_search_match(&mut self) {
        if let Some(ref query) = self.search_query {
            if query.is_empty() {
                return;
            }
            let q = query.to_lowercase();
            let current_node = self.current_node();
            if let Some((idx, _)) = current_node
                .children
                .iter()
                .enumerate()
                .find(|(_, child)| child.name.to_lowercase().contains(&q))
            {
                self.nav.selected_index = idx;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // If search mode is active, handle typing
        if self.search_query.is_some() {
            match key.code {
                KeyCode::Esc => {
                    self.cancel_search();
                }
                KeyCode::Enter => {
                    self.confirm_search();
                }
                KeyCode::Backspace => {
                    self.backspace_search();
                }
                KeyCode::Char(c) => {
                    self.update_search(c);
                }
                _ => {}
            }
            return;
        }

        // If sudo password modal is active, delegate keys to sudo modal
        if let Some(mut state) = self.sudo_modal_state.take() {
            match state.handle_key(key) {
                SudoAction::Pending => {
                    self.sudo_modal_state = Some(state);
                }
                SudoAction::Cancelled => {
                    if state.target_index.is_none() {
                        let _ = self.execute_bulldozer(None);
                    } else {
                        self.status_message = Some((self.language.demolish_cancelled().into(), false));
                    }
                    self.sudo_modal_state = None;
                }
                SudoAction::Submit(password) => {
                    if let Some(selected_idx) = state.target_index {
                        let target_name = state.target_name.clone();

                        let purge_action = {
                            let curr = self.current_node();
                            if selected_idx < curr.children.len() {
                                curr.children[selected_idx].purge_action.clone()
                            } else {
                                None
                            }
                        };

                        let res = if let Some(action) = purge_action {
                            action.execute_with_sudo(&password)
                        } else {
                            Err("Không có thao tác dọn dẹp nào".to_string())
                        };

                        match res {
                            Ok(()) => {
                                let curr = self.current_node_mut();
                                let freed = if selected_idx < curr.children.len() {
                                    let removed = curr.children.remove(selected_idx);
                                    curr.size = curr.size.saturating_sub(removed.size);
                                    removed.size
                                } else {
                                    state.target_size
                                };
                                let remaining_count = curr.children.len();
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
                                self.sudo_modal_state = None;
                            }
                            Err(e) => {
                                state.error_message = Some(e);
                                state.password_buffer.clear();
                                self.sudo_modal_state = Some(state);
                            }
                        }
                    } else {
                        // Bulldozer batch purge with sudo password
                        match self.execute_bulldozer(Some(&password)) {
                            Ok(_) => {
                                self.sudo_modal_state = None;
                            }
                            Err(e) => {
                                state.error_message = Some(e.to_string());
                                state.password_buffer.clear();
                                self.sudo_modal_state = Some(state);
                            }
                        }
                    }
                }
            }
            return;
        }

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
            KeyCode::Char('x')
            | KeyCode::Char('X')
            | KeyCode::Char('d')
            | KeyCode::Char('D')
            | KeyCode::Delete => {
                if let Some(child) = self.current_selected_child() {
                    let requires_sudo = child.requires_sudo
                        || child
                            .purge_action
                            .as_ref()
                            .map(|a| a.requires_sudo())
                            .unwrap_or(false);

                    if child.is_system_protected {
                        let notice = if self.is_in_scrapyard() {
                            if child.name.to_lowercase().contains("snap") {
                                if self.language == crate::i18n::Language::Vi {
                                    "Yêu cầu quyền sudo: Hãy chạy lệnh 'sudo snap remove <pkg> --revision <rev>' trong terminal.".to_string()
                                } else {
                                    "Sudo required: Run 'sudo snap remove <pkg> --revision <rev>' in terminal.".to_string()
                                }
                            } else if child.name.to_lowercase().contains("apt") {
                                if self.language == crate::i18n::Language::Vi {
                                    "Yêu cầu quyền sudo: Hãy chạy lệnh 'sudo apt-get clean' trong terminal.".to_string()
                                } else {
                                    "Sudo required: Run 'sudo apt-get clean' in terminal."
                                        .to_string()
                                }
                            } else {
                                self.language.protected_rejection_notice().to_string()
                            }
                        } else {
                            self.language.protected_rejection_notice().to_string()
                        };
                        self.status_message = Some((notice, true));
                    } else if requires_sudo {
                        self.sudo_modal_state = Some(SudoModalState::new_with_index(
                            Some(self.nav.selected_index),
                            child.name.clone(),
                            child.size,
                        ));
                    } else {
                        self.demolish_state = Some(DemolishState::new(child));
                    }
                }
            }
            KeyCode::Char('L') => {
                self.language = self.language.toggle();
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                self.open_current_selected();
            }
            KeyCode::Char('/') => {
                self.start_search();
            }
            KeyCode::Char('j') | KeyCode::Char('J') => {
                self.toggle_scrapyard();
            }
            KeyCode::Char('c') | KeyCode::Char('C') if self.is_in_scrapyard() => {
                let has_sudo = self.root_node.children.iter().any(|c| c.requires_sudo);
                if has_sudo {
                    let total_elevated: u64 = self.root_node.children.iter()
                        .filter(|c| c.requires_sudo)
                        .map(|c| c.size)
                        .sum();
                    let name = if self.language == crate::i18n::Language::Vi {
                        "San phẳng toàn bộ (Bulldozer All)".to_string()
                    } else {
                        "Bulldozer All (Batch Purge)".to_string()
                    };
                    self.sudo_modal_state = Some(SudoModalState::new_with_index(
                        None,
                        name,
                        total_elevated,
                    ));
                } else {
                    let _ = self.execute_bulldozer(None);
                }
            }
            _ => {}
        }
    }
}

pub fn launch_default_app(path: &std::path::Path) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .creation_flags(0x00000008)
            .spawn()?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        Ok(())
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

    #[test]
    fn test_spotlight_search_filtering_and_jumping() {
        let tree = create_mock_tree(); // root has "src" (idx 0) and "docs" (idx 1)
        let mut app = App::new(tree, ScannerOptions::default());

        assert_eq!(app.nav.selected_index, 0);
        assert!(app.search_query.is_none());

        // Start search
        app.start_search();
        assert_eq!(app.search_query, Some(String::new()));

        // Type 'd' -> matches "docs" (idx 1)
        app.update_search('d');
        assert_eq!(app.search_query, Some("d".to_string()));
        assert_eq!(app.nav.selected_index, 1);

        // Type 'o' -> matches "docs"
        app.update_search('o');
        assert_eq!(app.nav.selected_index, 1);

        // Backspace 'o'
        app.backspace_search();
        assert_eq!(app.search_query, Some("d".to_string()));

        // Confirm search
        app.confirm_search();
        assert!(app.search_query.is_none());
        assert_eq!(app.nav.selected_index, 1);

        // Start and Cancel search restores previous index
        app.start_search();
        app.update_search('s'); // jumps to "src" (idx 0)
        assert_eq!(app.nav.selected_index, 0);
        app.cancel_search();
        assert!(app.search_query.is_none());
        assert_eq!(app.nav.selected_index, 1); // restored back to docs!
    }

    #[test]
    fn test_open_current_selected_feedback() {
        let tree = create_mock_tree();
        let mut app = App::new(tree, ScannerOptions::default());

        // Call open on selected item
        let _ = app.open_current_selected();
        assert!(app.status_message.is_some());
        let (msg, _) = app.status_message.unwrap();
        assert!(msg.contains("src"));
    }

    #[test]
    fn test_scrapyard_mode_transitions_and_bulldozer() {
        let tree = create_mock_tree();
        let mut app = App::new(tree, ScannerOptions::default());
        assert!(!app.is_in_scrapyard());

        // Toggle into Scrapyard
        assert!(app.toggle_scrapyard());
        assert!(app.is_in_scrapyard());
        assert!(app.root_node.name.contains("SCRAPYARD"));

        // Bulldozer purge executes cleanly
        let res = app.execute_bulldozer(None);
        assert!(res.is_ok());

        // Navigate up exits back to original filesystem tree
        assert!(app.navigate_up());
        assert!(!app.is_in_scrapyard());
        assert_eq!(app.root_node.name, "root");
    }

    #[test]
    fn test_x_key_demolition_trigger() {
        let child = BuildingNode::new(
            "junk.tmp".into(),
            PathBuf::from("/tmp/junk.tmp"),
            false,
            false,
            1024,
            vec![],
        );
        let root = BuildingNode::new(
            "root".into(),
            PathBuf::from("/"),
            true,
            false,
            1024,
            vec![child],
        );
        let mut app = App::new(root, ScannerOptions::default());

        let key_x = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_key(key_x);
        assert!(
            app.demolish_state.is_some(),
            "Pressing 'x' must open demolition state"
        );
    }

    #[test]
    fn test_scrapyard_sudo_rejection_notice() {
        let mut protected_snap = BuildingNode::new(
            "Snap Inactive Revisions".into(),
            PathBuf::from("//scrapyard/snap"),
            false,
            false,
            5000,
            vec![],
        );
        protected_snap.is_system_protected = true;

        let root = BuildingNode::new(
            "ĐẠI LỘ PHẾ LIỆU (SCRAPYARD)".into(),
            PathBuf::from("//scrapyard"),
            true,
            false,
            5000,
            vec![protected_snap],
        );
        let mut app = App::new(root, ScannerOptions::default());
        app.saved_root_node = Some(BuildingNode::new(
            "root".into(),
            PathBuf::from("/"),
            true,
            false,
            0,
            vec![],
        ));

        let key_x = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_key(key_x);
        assert!(app.demolish_state.is_none());
        assert!(app.status_message.is_some());
        let msg = app.status_message.unwrap().0;
        assert!(
            msg.contains("sudo snap remove"),
            "Notice must contain exact terminal command: {}",
            msg
        );
    }

    #[test]
    fn test_scrapyard_sudo_modal_opens_and_submits() {
        let temp_dir = std::env::temp_dir().join("test_app_sudo_modal_dir");
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut sudo_junk = BuildingNode::new(
            "Apt Cache".into(),
            temp_dir.clone(),
            true,
            false,
            3000,
            vec![],
        );
        sudo_junk.is_system_protected = false;
        sudo_junk.requires_sudo = true;
        sudo_junk.purge_action = Some(crate::cleaner::PurgeAction::DirectoryDelete(
            temp_dir.clone(),
        ));

        let root = BuildingNode::new(
            "ĐẠI LỘ PHẾ LIỆU (SCRAPYARD)".into(),
            PathBuf::from("//scrapyard"),
            true,
            false,
            3000,
            vec![sudo_junk],
        );
        let mut app = App::new(root, ScannerOptions::default());
        app.saved_root_node = Some(BuildingNode::new(
            "root".into(),
            PathBuf::from("/"),
            true,
            false,
            0,
            vec![],
        ));

        // Press 'd'
        let key_d = KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_key(key_d);

        // Sudo modal must open, not blocked with notice
        assert!(app.demolish_state.is_none());
        assert!(app.sudo_modal_state.is_some());
        let modal = app.sudo_modal_state.as_ref().unwrap();
        assert_eq!(modal.target_name, "Apt Cache");

        // Type 'a', 'b', 'c'
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            });
        }
        assert_eq!(
            app.sudo_modal_state.as_ref().unwrap().password_buffer,
            "abc"
        );

        // Press Enter (DirectoryDelete succeeds with dummy password)
        let key_enter = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_key(key_enter);

        // Modal should close on success
        assert!(app.sudo_modal_state.is_none());
        assert_eq!(app.root_node.children.len(), 0);
        assert_eq!(app.root_node.size, 0);
        assert!(app.status_message.is_some());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_scrapyard_sudo_modal_cancel() {
        let mut sudo_junk = BuildingNode::new(
            "Snap Debris".into(),
            PathBuf::from("//scrapyard/snap"),
            false,
            false,
            4000,
            vec![],
        );
        sudo_junk.is_system_protected = false;
        sudo_junk.requires_sudo = true;

        let root = BuildingNode::new(
            "ĐẠI LỘ PHẾ LIỆU (SCRAPYARD)".into(),
            PathBuf::from("//scrapyard"),
            true,
            false,
            4000,
            vec![sudo_junk],
        );
        let mut app = App::new(root, ScannerOptions::default());

        // Press 'x'
        let key_x = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_key(key_x);
        assert!(app.sudo_modal_state.is_some());

        // Press Esc to cancel
        let key_esc = KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_key(key_esc);
        assert!(app.sudo_modal_state.is_none());
        assert!(app.status_message.is_some());
    }

    #[test]
    fn test_scrapyard_bulldozer_prompts_sudo_modal_when_elevated_targets_exist() {
        let mut elevated_target = BuildingNode::new(
            "Snap Inactive Revisions".into(),
            PathBuf::from("/var/lib/snapd/snaps/snap1.snap"),
            false,
            false,
            5000,
            vec![],
        );
        elevated_target.requires_sudo = true;
        elevated_target.is_system_protected = false;
        elevated_target.purge_action = Some(crate::cleaner::PurgeAction::Command("echo".into(), vec!["test".into()]));

        let root = BuildingNode::new(
            "ĐẠI LỘ PHẾ LIỆU (SCRAPYARD)".into(),
            PathBuf::from("//scrapyard"),
            true,
            false,
            5000,
            vec![elevated_target],
        );
        let mut app = App::new(root, ScannerOptions::default());
        assert!(app.is_in_scrapyard());

        let key_c = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_key(key_c);

        assert!(app.sudo_modal_state.is_some(), "Bulldozer with elevated targets must trigger sudo modal");
        let modal = app.sudo_modal_state.as_ref().unwrap();
        assert!(modal.target_index.is_none(), "Bulldozer sudo modal has None target_index");
    }
}
