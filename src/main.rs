use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use disk_skyline::app::App;
use disk_skyline::model;
use disk_skyline::scanner::{scan_directory, ScannerOptions, DEFAULT_MAX_DEPTH};
use disk_skyline::ui;
#[derive(Parser, Debug)]
#[command(
    name = "disk-skyline",
    version = "0.1.0",
    about = "Transform filesystem disk storage into an interactive terminal urban skyline"
)]
pub struct Cli {
    /// Target directory to survey (defaults to current directory '.')
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Maximum initial recursive scan depth
    #[arg(short, long, default_value_t = DEFAULT_MAX_DEPTH)]
    pub depth: usize,

    /// Print a single non-interactive skyline snapshot and exit
    #[arg(long)]
    pub snapshot: bool,

    /// Interface language / Ngôn ngữ hiển thị (vi | en)
    #[arg(short, long, value_enum)]
    pub lang: Option<disk_skyline::i18n::Language>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let target_path = if cli.path.is_relative() {
        std::env::current_dir()?.join(&cli.path)
    } else {
        cli.path.clone()
    };

    let canonical_path = target_path.canonicalize().unwrap_or(target_path);

    let language = cli.lang.unwrap_or_default();

    println!("============================================================");
    println!(" 🏙️  {}", language.app_title());
    println!("============================================================");
    println!(
        "{}",
        language.scanning_target(&canonical_path.to_string_lossy())
    );
    println!("{}", language.max_depth_info(cli.depth));
    println!("{}", language.wait_moment());
    let options = ScannerOptions {
        max_depth: cli.depth,
        sort_mode: model::SortMode::SizeDesc,
    };

    let root_node = match scan_directory(&canonical_path, &options) {
        Ok(node) => node,
        Err(e) => {
            eprintln!("\n[!] Không thể quét thư mục: {}", e);
            std::process::exit(1);
        }
    };

    // Setup terminal
    let mut app = App::new(root_node, options).with_language(language);

    // If snapshot flag is passed, render a single frame to stdout and exit
    if cli.snapshot {
        let (width, height) = crossterm::terminal::size().unwrap_or((100, 24));
        let width = width.max(80);
        let height = height.max(20);

        let backend = ratatui::backend::TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| ui::draw_ui(&mut app, f))?;
        let buf = terminal.backend().buffer();

        for y in 0..height {
            let mut line = String::new();
            for x in 0..width {
                line.push_str(buf[(x, y)].symbol());
            }
            println!("{}", line);
        }
        return Ok(());
    }

    // Setup terminal for interactive TUI
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Main TUI render loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("[!] Lỗi ứng dụng: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::draw_ui(app, f))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }
    }
    Ok(())
}
