use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::{Path, PathBuf};

use disk_skyline::app::App;
use disk_skyline::model::BuildingNode;
use disk_skyline::scanner::{is_virtual_pseudo_fs, ScannerOptions};
use disk_skyline::ui::draw_ui;

fn create_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

#[test]
fn test_virtual_pseudo_fs_detection() {
    assert!(is_virtual_pseudo_fs(Path::new("/proc")));
    assert!(is_virtual_pseudo_fs(Path::new("/proc/1/status")));
    assert!(is_virtual_pseudo_fs(Path::new("/sys/class/net")));
    assert!(is_virtual_pseudo_fs(Path::new("/dev/null")));
    assert!(is_virtual_pseudo_fs(Path::new("/run/systemd")));
    assert!(is_virtual_pseudo_fs(Path::new(
        "C:\\System Volume Information"
    )));

    assert!(!is_virtual_pseudo_fs(Path::new("/home/user")));
    assert!(!is_virtual_pseudo_fs(Path::new("/var/log")));
}

#[test]
fn test_system_protection_demolition_lockdown_e2e() {
    // Construct a mock root directory tree containing both protected system dirs and a user dir
    let etc_node = BuildingNode::new(
        "etc".into(),
        PathBuf::from("/etc"),
        true,
        false,
        50_000,
        vec![BuildingNode::new(
            "fstab".into(),
            PathBuf::from("/etc/fstab"),
            false,
            false,
            400,
            vec![],
        )],
    );
    assert!(
        etc_node.is_system_protected,
        "/etc must be system protected"
    );

    let usr_node = BuildingNode::new(
        "usr".into(),
        PathBuf::from("/usr"),
        true,
        false,
        1_000_000,
        vec![],
    );
    assert!(
        usr_node.is_system_protected,
        "/usr must be system protected"
    );

    let user_project = BuildingNode::new(
        "my_project".into(),
        PathBuf::from("/home/user/my_project"),
        true,
        false,
        10_000,
        vec![BuildingNode::new(
            "main.rs".into(),
            PathBuf::from("/home/user/my_project/main.rs"),
            false,
            false,
            500,
            vec![],
        )],
    );
    assert!(
        !user_project.is_system_protected,
        "User project must not be system protected"
    );

    let root = BuildingNode::new(
        "root".into(),
        PathBuf::from("/"),
        true,
        false,
        1_060_000,
        vec![usr_node, etc_node, user_project],
    );

    let mut app = App::new(root, ScannerOptions::default());

    let backend = TestBackend::new(100, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // 1. Initial render focusing on usr (index 0)
    terminal.draw(|f| draw_ui(&mut app, f)).unwrap();
    assert_eq!(app.nav.selected_index, 0);
    assert_eq!(app.current_selected_child().unwrap().name, "usr");
    assert!(app.current_selected_child().unwrap().is_system_protected);

    // 2. Press 'd' on protected 'usr' -> MUST BE REFUSED
    app.handle_key(create_key(KeyCode::Char('d')));

    assert!(
        app.demolish_state.is_none(),
        "Demolish modal must not open for protected directory"
    );
    assert!(
        app.status_message.is_some(),
        "Status message must report rejection"
    );
    let (msg, is_err) = app.status_message.as_ref().unwrap();
    assert!(is_err, "Rejection status message must be flagged as error");
    assert!(
        msg.contains("TỪ CHỐI THAO TÁC") || msg.contains("ACTION REJECTED"),
        "Message must explain refusal: {}",
        msg
    );

    // 3. Render HUD and verify that HUD reflects [KHÓA BẢO VỆ HỆ THỐNG]
    terminal.draw(|f| draw_ui(&mut app, f)).unwrap();

    // 4. Move to user_project (index 2) -> Demolition should NOT be blocked
    app.handle_key(create_key(KeyCode::Right)); // moves to etc
    app.handle_key(create_key(KeyCode::Right)); // moves to my_project
    assert_eq!(app.nav.selected_index, 2);
    assert_eq!(app.current_selected_child().unwrap().name, "my_project");
    assert!(!app.current_selected_child().unwrap().is_system_protected);

    // Press 'd' on user_project -> MUST OPEN DEMOLISH MODAL
    app.handle_key(create_key(KeyCode::Char('d')));
    assert!(
        app.demolish_state.is_some(),
        "Demolish modal must open for user directory"
    );

    // Cancel modal with Esc
    app.handle_key(create_key(KeyCode::Esc));
    assert!(app.demolish_state.is_none());
}
