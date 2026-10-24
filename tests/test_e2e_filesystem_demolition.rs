use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::fs;
use std::path::PathBuf;

use disk_skyline::app::App;
use disk_skyline::scanner::{scan_directory, ScannerOptions};
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
fn test_e2e_filesystem_scan_navigation_and_demolition() {
    // Strictly isolate test filesystem operations inside this repo's target directory
    let repo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sandbox = repo_dir.join("target/e2e_test_sandbox");

    // Clean up any stale directory from previous runs
    if sandbox.exists() {
        let _ = fs::remove_dir_all(&sandbox);
    }
    fs::create_dir_all(&sandbox).expect("Failed to create sandbox directory");

    // Populate sandbox with real test files and directories
    let doomed_file = sandbox.join("doomed_artifact.log");
    fs::write(
        &doomed_file,
        "Sample log file data for demolition testing\n",
    )
    .unwrap();

    let sub_dir = sandbox.join("sub_module");
    fs::create_dir_all(&sub_dir).unwrap();
    let sub_file1 = sub_dir.join("config.json");
    fs::write(&sub_file1, r#"{"mode": "test"}"#).unwrap();
    let sub_file2 = sub_dir.join("module.rs");
    fs::write(&sub_file2, "pub fn module_run() {}\n").unwrap();

    let doomed_folder = sandbox.join("temporary_cache");
    fs::create_dir_all(&doomed_folder).unwrap();
    fs::write(doomed_folder.join("cache.1"), "cache data 1").unwrap();
    fs::write(doomed_folder.join("cache.2"), "cache data 2").unwrap();

    assert!(
        doomed_file.exists(),
        "Doomed file must exist on disk prior to test"
    );
    assert!(
        doomed_folder.exists(),
        "Doomed folder must exist on disk prior to test"
    );

    // --- PHASE 1: Real Disk Scan ---
    let options = ScannerOptions::default();
    let root_node = scan_directory(&sandbox, &options).expect("Disk scan must succeed");

    assert_eq!(
        root_node.children.len(),
        3,
        "Expected 3 top-level items in sandbox"
    );
    let original_total_size = root_node.size;
    assert!(original_total_size > 0, "Sandbox total size must be > 0");

    // --- PHASE 2: TUI App Initialization & Navigation ---
    let mut app = App::new(root_node, options);
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render initial view
    terminal.draw(|f| draw_ui(&mut app, f)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let screen_text: String = buffer.content().iter().map(|c| c.symbol()).collect();
    assert!(
        screen_text.contains("DISK SKYLINE"),
        "HUD title must be rendered"
    );

    // --- PHASE 3: Demolish a single file using 'x' hotkey ---
    // Find index of doomed_file
    let doomed_file_name = "doomed_artifact.log";
    let file_idx = app
        .root_node
        .children
        .iter()
        .position(|c| c.name == doomed_file_name)
        .expect("doomed_artifact.log must be present in root children");

    app.nav.selected_index = file_idx;
    assert_eq!(app.current_selected_child().unwrap().name, doomed_file_name);

    // 1. Press 'x' to open demolition modal
    app.handle_key(create_key(KeyCode::Char('x')));
    assert!(
        app.demolish_state.is_some(),
        "Pressing 'x' must open the DemolishState modal"
    );

    // Render with modal open
    terminal.draw(|f| draw_ui(&mut app, f)).unwrap();
    let modal_screen = terminal.backend().buffer().clone();
    let modal_text: String = modal_screen.content().iter().map(|c| c.symbol()).collect();
    assert!(
        modal_text.contains("PHÁ DỠ")
            || modal_text.contains("DEMOLITION")
            || modal_text.contains("XÁC NHẬN"),
        "Demolition modal must be visually rendered"
    );

    // 2. Confirm demolition with 'y'
    app.handle_key(create_key(KeyCode::Char('y')));
    assert!(
        app.demolish_state.is_none(),
        "Modal must close after confirmation"
    );

    // 3. Verify actual disk deletion
    assert!(
        !doomed_file.exists(),
        "doomed_artifact.log MUST be deleted from disk"
    );

    // 4. Verify in-memory tree was updated without full rescan
    assert!(
        app.root_node
            .children
            .iter()
            .all(|c| c.name != doomed_file_name),
        "doomed_artifact.log must be pruned from parent children"
    );
    assert!(
        app.root_node.size < original_total_size,
        "Parent size must decrease after demolition"
    );
    assert!(
        app.status_message.is_some(),
        "Status message must report successful demolition"
    );

    // --- PHASE 4: Demolish an entire directory using 'Delete' key ---
    let folder_name = "temporary_cache";
    let folder_idx = app
        .root_node
        .children
        .iter()
        .position(|c| c.name == folder_name)
        .expect("temporary_cache folder must be present in root children");

    app.nav.selected_index = folder_idx;
    assert_eq!(app.current_selected_child().unwrap().name, folder_name);

    // Press 'Delete' key
    app.handle_key(create_key(KeyCode::Delete));
    assert!(app.demolish_state.is_some());

    // Confirm with 'y'
    app.handle_key(create_key(KeyCode::Char('y')));
    assert!(app.demolish_state.is_none());

    // Verify directory is deleted from disk
    assert!(
        !doomed_folder.exists(),
        "temporary_cache folder MUST be deleted from disk"
    );
    assert!(
        app.root_node.children.iter().all(|c| c.name != folder_name),
        "temporary_cache folder must be pruned from memory"
    );

    // --- PHASE 5: Drill-down and Navigation in remaining directory ---
    assert_eq!(app.root_node.children.len(), 1);
    assert_eq!(app.root_node.children[0].name, "sub_module");
    app.nav.selected_index = 0;

    // Enter directory
    assert!(app.drill_down());
    assert_eq!(app.breadcrumbs(), vec!["e2e_test_sandbox", "sub_module"]);
    assert_eq!(app.current_node().children.len(), 2);

    // Navigate back up
    assert!(app.navigate_up());
    assert_eq!(app.breadcrumbs(), vec!["e2e_test_sandbox"]);

    // --- CLEANUP: Safely remove the sandbox directory from repo ---
    let _ = fs::remove_dir_all(&sandbox);
    assert!(!sandbox.exists(), "Sandbox directory must be cleaned up");
}

#[test]
fn test_e2e_cli_binary_execution_on_repo_sandbox() {
    use std::process::Command;

    let repo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sandbox = repo_dir.join("target/e2e_cli_sandbox");

    if sandbox.exists() {
        let _ = fs::remove_dir_all(&sandbox);
    }
    fs::create_dir_all(&sandbox).expect("Failed to create CLI sandbox");

    // Create test files
    fs::write(sandbox.join("service.rs"), vec![b'a'; 15_000]).unwrap();
    fs::write(sandbox.join("archive.zip"), vec![b'z'; 40_000]).unwrap();
    fs::write(sandbox.join("script.py"), vec![b'p'; 8_000]).unwrap();

    let bin_path = repo_dir.join("target/release/disk-skyline");
    assert!(
        bin_path.exists(),
        "Compiled binary target/release/disk-skyline must exist"
    );

    let output = Command::new(&bin_path)
        .arg(&sandbox)
        .arg("--snapshot")
        .output()
        .expect("Failed to execute disk-skyline binary");

    assert!(
        output.status.success(),
        "CLI snapshot must exit successfully"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("DISK SKYLINE"),
        "Stdout must contain DISK SKYLINE header"
    );
    assert!(
        stdout.contains("service.rs"),
        "Stdout must display service.rs building"
    );
    assert!(
        stdout.contains("archive.zip"),
        "Stdout must display archive.zip building"
    );

    // Clean up sandbox
    let _ = fs::remove_dir_all(&sandbox);
    assert!(!sandbox.exists());
}
