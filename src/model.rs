use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingCategory {
    Directory,
    Code,
    Media,
    Archive,
    Binary,
    Document,
    Restricted,
    Other,
}

impl BuildingCategory {
    pub fn from_path(path: &Path, is_dir: bool, is_hidden: bool) -> Self {
        if is_hidden {
            return BuildingCategory::Restricted;
        }

        if is_dir {
            return BuildingCategory::Directory;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "rs" | "go" | "py" | "js" | "ts" | "jsx" | "tsx" | "c" | "cpp" | "h" | "hpp"
            | "java" | "kt" | "swift" | "html" | "css" | "scss" | "json" | "yaml" | "yml"
            | "toml" | "sh" | "bash" | "zsh" | "sql" | "md" | "xml" | "lua" => {
                BuildingCategory::Code
            }
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico" | "mp4" | "mkv" | "avi"
            | "mov" | "webm" | "mp3" | "flac" | "wav" | "ogg" | "m4a" => BuildingCategory::Media,
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" | "tgz" => {
                BuildingCategory::Archive
            }
            "exe" | "bin" | "o" | "a" | "so" | "dylib" | "dll" | "wasm" => BuildingCategory::Binary,
            "pdf" | "doc" | "docx" | "txt" | "rtf" | "epub" | "odt" => BuildingCategory::Document,
            _ => BuildingCategory::Other,
        }
    }

    pub fn display_label(&self) -> &'static str {
        self.display_label_lang(crate::i18n::Language::Vi)
    }

    pub fn display_label_lang(&self, lang: crate::i18n::Language) -> &'static str {
        match (lang, self) {
            (crate::i18n::Language::Vi, BuildingCategory::Directory) => "Văn phòng (Thư mục)",
            (crate::i18n::Language::En, BuildingCategory::Directory) => "Office (Directory)",
            (crate::i18n::Language::Vi, BuildingCategory::Code) => "Trung tâm Công nghệ (Code)",
            (crate::i18n::Language::En, BuildingCategory::Code) => "Tech Center (Code)",
            (crate::i18n::Language::Vi, BuildingCategory::Media) => "Khu Giải trí & Media",
            (crate::i18n::Language::En, BuildingCategory::Media) => "Entertainment & Media",
            (crate::i18n::Language::Vi, BuildingCategory::Archive) => "Kho Vận & Nén (Archive)",
            (crate::i18n::Language::En, BuildingCategory::Archive) => "Storage & Archive",
            (crate::i18n::Language::Vi, BuildingCategory::Binary) => "Nhà máy Thực thi (Binary)",
            (crate::i18n::Language::En, BuildingCategory::Binary) => "Industrial Plant (Binary)",
            (crate::i18n::Language::Vi, BuildingCategory::Document) => "Thư viện & Văn bản",
            (crate::i18n::Language::En, BuildingCategory::Document) => "Library & Documents",
            (crate::i18n::Language::Vi, BuildingCategory::Restricted) => "Khu vực cấm (Bunker/Ẩn)",
            (crate::i18n::Language::En, BuildingCategory::Restricted) => {
                "Restricted Bunker (Hidden)"
            }
            (crate::i18n::Language::Vi, BuildingCategory::Other) => "Công trình Dân dụng",
            (crate::i18n::Language::En, BuildingCategory::Other) => "Civic / General File",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    SizeDesc,
    SizeAsc,
    NameAsc,
    Category,
}

impl SortMode {
    pub fn next(&self) -> Self {
        match self {
            SortMode::SizeDesc => SortMode::SizeAsc,
            SortMode::SizeAsc => SortMode::NameAsc,
            SortMode::NameAsc => SortMode::Category,
            SortMode::Category => SortMode::SizeDesc,
        }
    }

    pub fn label(&self) -> &'static str {
        self.label_lang(crate::i18n::Language::Vi)
    }

    pub fn label_lang(&self, lang: crate::i18n::Language) -> &'static str {
        match (lang, self) {
            (crate::i18n::Language::Vi, SortMode::SizeDesc) => "Dung lượng (Cao -> Thấp)",
            (crate::i18n::Language::En, SortMode::SizeDesc) => "Size (High -> Low)",
            (crate::i18n::Language::Vi, SortMode::SizeAsc) => "Dung lượng (Thấp -> Cao)",
            (crate::i18n::Language::En, SortMode::SizeAsc) => "Size (Low -> High)",
            (crate::i18n::Language::Vi, SortMode::NameAsc) => "Tên (A -> Z)",
            (crate::i18n::Language::En, SortMode::NameAsc) => "Name (A -> Z)",
            (crate::i18n::Language::Vi, SortMode::Category) => "Phân khu Đô thị",
            (crate::i18n::Language::En, SortMode::Category) => "Zoning Category",
        }
    }
}
#[derive(Debug, Clone)]
pub struct BuildingNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub is_system_protected: bool,
    pub size: u64,
    pub file_type: BuildingCategory,
    pub children: Vec<BuildingNode>,
}

impl BuildingNode {
    pub fn new(
        name: String,
        path: PathBuf,
        is_dir: bool,
        is_hidden: bool,
        size: u64,
        children: Vec<BuildingNode>,
    ) -> Self {
        let is_system_protected = is_os_critical_path(&path);
        let file_type = BuildingCategory::from_path(&path, is_dir, is_hidden);
        Self {
            name,
            path,
            is_dir,
            is_hidden,
            is_system_protected,
            size,
            file_type,
            children,
        }
    }

    pub fn new_with_protection(
        name: String,
        path: PathBuf,
        is_dir: bool,
        is_hidden: bool,
        is_system_protected: bool,
        size: u64,
        children: Vec<BuildingNode>,
    ) -> Self {
        let file_type = BuildingCategory::from_path(&path, is_dir, is_hidden);
        Self {
            name,
            path,
            is_dir,
            is_hidden,
            is_system_protected,
            size,
            file_type,
            children,
        }
    }

    pub fn item_count(&self) -> usize {
        if self.is_dir {
            self.children.len()
        } else {
            1
        }
    }

    pub fn formatted_size(&self) -> String {
        format_bytes(self.size)
    }

    pub fn sort_children(&mut self, mode: SortMode) {
        match mode {
            SortMode::SizeDesc => {
                self.children.sort_by_key(|a| std::cmp::Reverse(a.size));
            }
            SortMode::SizeAsc => {
                self.children.sort_by_key(|a| a.size);
            }
            SortMode::NameAsc => {
                self.children.sort_by_key(|a| a.name.to_lowercase());
            }
            SortMode::Category => {
                self.children.sort_by(|a, b| {
                    (a.file_type as u8)
                        .cmp(&(b.file_type as u8))
                        .then_with(|| b.size.cmp(&a.size))
                });
            }
        }
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn is_os_critical_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let trimmed = path_str.trim();

    if trimmed == "/" || trimmed == "\\" || (trimmed.len() == 3 && trimmed.ends_with(":\\")) {
        return true;
    }

    let lower = trimmed.to_lowercase();
    if lower.ends_with("pagefile.sys")
        || lower.ends_with("hiberfil.sys")
        || lower.ends_with("swapfile.sys")
        || lower.ends_with("dumpstack.log")
    {
        return true;
    }
    if lower.contains("c:\\windows")
        || lower.contains("c:/windows")
        || lower.contains("c:\\program files")
        || lower.contains("c:/program files")
        || lower.contains("c:\\programdata")
        || lower.contains("c:/programdata")
        || lower.contains("system volume information")
        || lower.contains("recovery")
    {
        return true;
    }

    const UNIX_PROTECTED_ROOTS: &[&str] = &[
        "/bin", "/sbin", "/usr", "/etc", "/boot", "/root", "/lib", "/lib64",
    ];

    for &prefix in UNIX_PROTECTED_ROOTS {
        if trimmed == prefix || trimmed.starts_with(&format!("{}/", prefix)) {
            return true;
        }
    }

    false
}

#[derive(Debug, Clone, Default)]
pub struct NavigationState {
    pub current_path_indices: Vec<usize>,
    pub selected_index: usize,
    pub viewport_offset_x: usize,
    pub sort_mode: SortMode,
}

impl NavigationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_left(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_right(&mut self, max_items: usize) {
        if max_items > 0 && self.selected_index + 1 < max_items {
            self.selected_index += 1;
        }
    }

    pub fn drill_down(&mut self, target_child_count: usize) -> bool {
        if target_child_count == 0 {
            return false;
        }
        self.current_path_indices.push(self.selected_index);
        self.selected_index = 0;
        self.viewport_offset_x = 0;
        true
    }

    pub fn navigate_up(&mut self) -> bool {
        if let Some(prev_index) = self.current_path_indices.pop() {
            self.selected_index = prev_index;
            self.viewport_offset_x = 0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_category_from_path() {
        // Arrange
        let hidden_path = Path::new(".git");
        let code_path = Path::new("main.rs");
        let media_path = Path::new("trailer.mp4");
        let archive_path = Path::new("backup.tar.gz");
        let dir_path = Path::new("src");

        // Act & Assert
        assert_eq!(
            BuildingCategory::from_path(hidden_path, true, true),
            BuildingCategory::Restricted
        );
        assert_eq!(
            BuildingCategory::from_path(code_path, false, false),
            BuildingCategory::Code
        );
        assert_eq!(
            BuildingCategory::from_path(media_path, false, false),
            BuildingCategory::Media
        );
        assert_eq!(
            BuildingCategory::from_path(archive_path, false, false),
            BuildingCategory::Archive
        );
        assert_eq!(
            BuildingCategory::from_path(dir_path, true, false),
            BuildingCategory::Directory
        );
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(2048), "2.00 KB");
        assert_eq!(format_bytes(10 * 1024 * 1024), "10.00 MB");
        assert_eq!(format_bytes(3 * 1024 * 1024 * 1024), "3.00 GB");
    }

    #[test]
    fn test_building_node_sorting() {
        // Arrange
        let mut parent = BuildingNode::new(
            "root".to_string(),
            PathBuf::from("/root"),
            true,
            false,
            1000,
            vec![
                BuildingNode::new(
                    "small.txt".into(),
                    PathBuf::from("small.txt"),
                    false,
                    false,
                    10,
                    vec![],
                ),
                BuildingNode::new(
                    "big.bin".into(),
                    PathBuf::from("big.bin"),
                    false,
                    false,
                    800,
                    vec![],
                ),
                BuildingNode::new(
                    "mid.rs".into(),
                    PathBuf::from("mid.rs"),
                    false,
                    false,
                    190,
                    vec![],
                ),
            ],
        );

        // Act: Sort descending by size
        parent.sort_children(SortMode::SizeDesc);

        // Assert
        assert_eq!(parent.children[0].name, "big.bin");
        assert_eq!(parent.children[1].name, "mid.rs");
        assert_eq!(parent.children[2].name, "small.txt");

        // Act: Sort ascending by name
        parent.sort_children(SortMode::NameAsc);
        assert_eq!(parent.children[0].name, "big.bin");
        assert_eq!(parent.children[1].name, "mid.rs");
        assert_eq!(parent.children[2].name, "small.txt");
    }

    #[test]
    fn test_navigation_state_transitions() {
        // Arrange
        let mut nav = NavigationState::new();

        // Act & Assert
        nav.move_right(5);
        assert_eq!(nav.selected_index, 1);

        assert!(nav.drill_down(3));
        assert_eq!(nav.selected_index, 0);
        assert_eq!(nav.current_path_indices, vec![1]);

        assert!(nav.navigate_up());
        assert_eq!(nav.selected_index, 1);
        assert!(nav.current_path_indices.is_empty());
    }

    #[test]
    fn test_is_os_critical_path() {
        assert!(is_os_critical_path(Path::new("/")));
        assert!(is_os_critical_path(Path::new("/etc")));
        assert!(is_os_critical_path(Path::new("/usr/bin")));
        assert!(is_os_critical_path(Path::new("/boot/vmlinuz")));
        assert!(is_os_critical_path(Path::new("C:\\Windows\\System32")));
        assert!(is_os_critical_path(Path::new("pagefile.sys")));
        assert!(is_os_critical_path(Path::new("C:\\hiberfil.sys")));

        assert!(!is_os_critical_path(Path::new("/home/user/project")));
        assert!(!is_os_critical_path(Path::new("/tmp/build")));
        assert!(!is_os_critical_path(Path::new("src/main.rs")));
    }
}
