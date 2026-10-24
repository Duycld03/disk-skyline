use clap::Parser;
use disk_skyline::app::App;
use disk_skyline::gui::{run_gui, window_conf};
use disk_skyline::i18n::Language;
use disk_skyline::scanner::{scan_directory, ScannerOptions, DEFAULT_MAX_DEPTH};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "disk-skyline-gui",
    version = "0.1.0",
    about = "2D Desktop Game Window Visualizer for Disk Skyline using Macroquad"
)]
struct GuiCli {
    /// Target directory to survey (defaults to current directory '.')
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Maximum initial recursive scan depth
    #[arg(short, long, default_value_t = DEFAULT_MAX_DEPTH)]
    depth: usize,

    /// Interface language / Ngôn ngữ hiển thị (vi | en)
    #[arg(short, long, value_enum)]
    pub lang: Option<Language>,

    /// Launch directly into the System Scrapyard survey (Đại Lộ Phế Liệu)
    #[arg(short = 's', long)]
    pub scrapyard: bool,
}

fn main() {
    let cli = GuiCli::parse();

    // Check display environment on Linux
    #[cfg(target_os = "linux")]
    {
        if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
            eprintln!(
                "Notice: No graphical display server detected (DISPLAY or WAYLAND_DISPLAY not set).\n\
                 For headless server environments, please use the terminal interface:\n  disk-skyline [PATH]"
            );
            return;
        }
    }

    let target_path = cli.path.canonicalize().unwrap_or_else(|_| cli.path.clone());

    let options = ScannerOptions {
        max_depth: cli.depth,
        sort_mode: disk_skyline::model::SortMode::default(),
    };

    let language = cli.lang.unwrap_or_else(Language::detect_system);

    let root_node = if cli.scrapyard {
        let targets = disk_skyline::cleaner::survey_all_system_junk();
        disk_skyline::model::BuildingNode::build_scrapyard_avenue(&targets)
    } else {
        match scan_directory(&target_path, &options) {
            Ok(node) => node,
            Err(e) => {
                eprintln!("Error scanning directory: {}", e);
                return;
            }
        }
    };

    let app = App::new(root_node, options).with_language(language);

    macroquad::Window::from_config(window_conf(), run_gui(app));
}
