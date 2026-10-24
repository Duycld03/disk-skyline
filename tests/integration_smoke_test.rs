use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

use disk_skyline::app::App;
use disk_skyline::model::{BuildingCategory, BuildingNode};
use disk_skyline::scanner::ScannerOptions;
use disk_skyline::ui::draw_ui;

fn create_test_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn build_mock_city() -> BuildingNode {
    let sub1 = BuildingNode::new(
        "main.rs".into(),
        PathBuf::from("src/main.rs"),
        false,
        false,
        15_000,
        vec![],
    );
    let sub2 = BuildingNode::new(
        "app.rs".into(),
        PathBuf::from("src/app.rs"),
        false,
        false,
        25_000,
        vec![],
    );
    let src_dir = BuildingNode::new(
        "src".into(),
        PathBuf::from("src"),
        true,
        false,
        40_000,
        vec![sub1, sub2],
    );

    let dot_git = BuildingNode::new(
        ".git".into(),
        PathBuf::from(".git"),
        true,
        true,
        500_000,
        vec![BuildingNode::new(
            "config".into(),
            PathBuf::from(".git/config"),
            false,
            false,
            300,
            vec![],
        )],
    );

    let movie = BuildingNode::new(
        "intro.mp4".into(),
        PathBuf::from("intro.mp4"),
        false,
        false,
        80_000_000,
        vec![],
    );
    let archive = BuildingNode::new(
        "backup.zip".into(),
        PathBuf::from("backup.zip"),
        false,
        false,
        15_000_000,
        vec![],
    );

    BuildingNode::new(
        "workspace".into(),
        PathBuf::from("/workspace"),
        true,
        false,
        95_540_000,
        vec![movie, archive, dot_git, src_dir],
    )
}

#[test]
fn test_end_to_end_tui_flow() {
    let tree = build_mock_city();
    let mut app = App::new(tree, ScannerOptions::default());

    // Setup TestBackend for ratatui (80 cols, 24 rows)
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // 1. Initial draw
    terminal.draw(|f| draw_ui(&mut app, f)).unwrap();
    let buffer = terminal.backend().buffer();

    assert_eq!(buffer[(1, 0)].symbol(), "D");
    assert_eq!(buffer[(2, 0)].symbol(), "I");

    // 2. Camera panning: move right across buildings
    app.handle_key(create_test_key(KeyCode::Right));
    assert_eq!(app.nav.selected_index, 1);

    app.handle_key(create_test_key(KeyCode::Right));
    assert_eq!(app.nav.selected_index, 2);

    // Selected building at index 2 is `.git` (Restricted bunker)
    let selected = app.current_selected_child().unwrap();
    assert_eq!(selected.name, ".git");
    assert_eq!(selected.file_type, BuildingCategory::Restricted);

    // Redraw with .git selected
    terminal.draw(|f| draw_ui(&mut app, f)).unwrap();

    // 3. Move to `src` (index 3) and drill down
    app.handle_key(create_test_key(KeyCode::Right));
    assert_eq!(app.nav.selected_index, 3);
    assert_eq!(app.current_selected_child().unwrap().name, "src");

    // Drill down
    app.handle_key(create_test_key(KeyCode::Enter));
    assert_eq!(app.breadcrumbs(), vec!["workspace", "src"]);
    assert_eq!(app.current_selected_child().unwrap().name, "main.rs");

    // Redraw interior street
    terminal.draw(|f| draw_ui(&mut app, f)).unwrap();

    // Navigate back to workspace
    app.handle_key(create_test_key(KeyCode::Esc));
    assert_eq!(app.breadcrumbs(), vec!["workspace"]);
    assert_eq!(app.nav.selected_index, 3);

    // 4. Test demolition dialog on `.git` (restricted -> requires YES)
    app.handle_key(create_test_key(KeyCode::Left)); // back to .git
    assert_eq!(app.nav.selected_index, 2);

    app.handle_key(create_test_key(KeyCode::Char('d')));
    assert!(app.demolish_state.is_some());
    let state = app.demolish_state.as_ref().unwrap();
    assert!(
        state.requires_explicit_yes,
        "Restricted dotfile must require explicit YES"
    );

    // Redraw modal
    terminal.draw(|f| draw_ui(&mut app, f)).unwrap();

    // Cancel demolition with Esc
    app.handle_key(create_test_key(KeyCode::Esc));
    assert!(app.demolish_state.is_none());

    // 5. Test demolition on standard file (movie: index 0)
    app.nav.selected_index = 0;
    app.handle_key(create_test_key(KeyCode::Char('d')));
    assert!(app.demolish_state.is_some());
    assert!(!app.demolish_state.as_ref().unwrap().requires_explicit_yes);

    // Cancel
    app.handle_key(create_test_key(KeyCode::Char('n')));
    assert!(app.demolish_state.is_none());

    // 6. Test sort toggle
    let initial_sort = app.nav.sort_mode;
    app.handle_key(create_test_key(KeyCode::Tab));
    assert_ne!(app.nav.sort_mode, initial_sort);

    // 6.1 Test runtime language toggle with 'L'
    let initial_lang = app.language;
    app.handle_key(create_test_key(KeyCode::Char('L')));
    assert_ne!(
        app.language, initial_lang,
        "Key 'L' must toggle active language"
    );
    terminal.draw(|f| draw_ui(&mut app, f)).unwrap();
    app.handle_key(create_test_key(KeyCode::Char('L')));
    assert_eq!(
        app.language, initial_lang,
        "Key 'L' must toggle language back"
    );
    // 7. Test quit
    app.handle_key(create_test_key(KeyCode::Char('q')));
    assert!(app.should_quit);
}
