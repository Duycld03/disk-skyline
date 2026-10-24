use clap::ValueEnum;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Language {
    #[value(name = "vi")]
    Vi,
    #[value(name = "en")]
    En,
}

impl Default for Language {
    fn default() -> Self {
        Self::detect_system()
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Vi => write!(f, "vi"),
            Language::En => write!(f, "en"),
        }
    }
}

impl Language {
    pub fn toggle(&self) -> Self {
        match self {
            Language::Vi => Language::En,
            Language::En => Language::Vi,
        }
    }

    pub fn detect_system() -> Self {
        for var_name in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(val) = std::env::var(var_name) {
                let lower = val.to_lowercase();
                if lower.starts_with("vi") {
                    return Language::Vi;
                }
            }
        }
        Language::En
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Vi => "Tiếng Việt",
            Language::En => "English",
        }
    }

    // UI & App Title
    pub fn app_title(&self) -> &'static str {
        match self {
            Language::Vi => "DISK SKYLINE - Khởi tạo quy hoạch đô thị",
            Language::En => "DISK SKYLINE - Urban Planning Initialized",
        }
    }

    pub fn scanning_target(&self, path: &str) -> String {
        match self {
            Language::Vi => format!("Đang khảo sát thực địa: {}", path),
            Language::En => format!("Surveying terrain: {}", path),
        }
    }

    pub fn max_depth_info(&self, depth: usize) -> String {
        match self {
            Language::Vi => format!("Độ sâu quét tối đa: {} tầng", depth),
            Language::En => format!("Maximum scan depth: {} stories", depth),
        }
    }

    pub fn wait_moment(&self) -> &'static str {
        match self {
            Language::Vi => "Vui lòng đợi trong giây lát...",
            Language::En => "Please wait a moment...",
        }
    }

    pub fn avenue_title(&self, name: &str) -> String {
        match self {
            Language::Vi => format!("DISK SKYLINE Đại lộ: {}", name),
            Language::En => format!("DISK SKYLINE Avenue: {}", name),
        }
    }

    pub fn planning_mode_label(&self, sort_str: &str, count: usize, size_str: &str) -> String {
        match self {
            Language::Vi => format!(
                "Quy hoạch: [{}] {} công trình (Tổng: {})",
                sort_str, count, size_str
            ),
            Language::En => format!(
                "Zoning: [{}] {} buildings (Total: {})",
                sort_str, count, size_str
            ),
        }
    }

    pub fn sort_name_desc(&self) -> &'static str {
        match self {
            Language::Vi => "Tên (A -> Z)",
            Language::En => "Name (A -> Z)",
        }
    }

    pub fn sort_size_desc(&self) -> &'static str {
        match self {
            Language::Vi => "Dung lượng (Cao -> Thấp)",
            Language::En => "Size (High -> Low)",
        }
    }

    pub fn sort_category_desc(&self) -> &'static str {
        match self {
            Language::Vi => "Phân khu đô thị",
            Language::En => "Zoning Category",
        }
    }

    pub fn selected_badge(&self) -> &'static str {
        match self {
            Language::Vi => "▲ [CHỌN]",
            Language::En => "▲ [SELECT]",
        }
    }

    pub fn street_label(&self) -> &'static str {
        match self {
            Language::Vi => "Phố: ",
            Language::En => "St: ",
        }
    }

    pub fn building_label(&self) -> &'static str {
        match self {
            Language::Vi => "CÔNG TRÌNH: ",
            Language::En => "BUILDING: ",
        }
    }

    pub fn size_label(&self) -> &'static str {
        match self {
            Language::Vi => "Dung lượng: ",
            Language::En => "Size: ",
        }
    }

    pub fn path_label(&self) -> &'static str {
        match self {
            Language::Vi => "Đường dẫn: ",
            Language::En => "Path: ",
        }
    }

    pub fn scale_label(&self, count: usize) -> String {
        match self {
            Language::Vi => format!("Quy mô: {} tầng (phần tử)", count),
            Language::En => format!("Scale: {} stories (items)", count),
        }
    }

    pub fn bunker_tag(&self) -> &'static str {
        match self {
            Language::Vi => "[!] KHU VỰC CẤM (BUNKER)",
            Language::En => "[!] RESTRICTED BUNKER",
        }
    }

    pub fn protected_tag(&self) -> &'static str {
        match self {
            Language::Vi => "[KHÓA BẢO VỆ HỆ THỐNG]",
            Language::En => "[SYSTEM PROTECTED LOCK]",
        }
    }

    pub fn protected_demolish_disabled_key(&self) -> &'static str {
        match self {
            Language::Vi => "[x/Del] Khóa hệ thống",
            Language::En => "[x/Del] System Locked",
        }
    }

    pub fn protected_rejection_notice(&self) -> &'static str {
        match self {
            Language::Vi => "[TỪ CHỐI THAO TÁC: Công trình hạ tầng thiết yếu của Hệ điều hành. Phá dỡ bị cấm tuyệt đối!]",
            Language::En => "[ACTION REJECTED: Essential Operating System infrastructure. Demolition strictly forbidden!]",
        }
    }

    // Controls hints
    pub fn nav_stroll(&self) -> &'static str {
        match self {
            Language::Vi => "Dạo phố",
            Language::En => "Stroll",
        }
    }

    pub fn nav_enter(&self) -> &'static str {
        match self {
            Language::Vi => "Vào trong",
            Language::En => "Enter",
        }
    }

    pub fn nav_back(&self) -> &'static str {
        match self {
            Language::Vi => "Ra ngoài",
            Language::En => "Back",
        }
    }

    pub fn nav_sort(&self) -> &'static str {
        match self {
            Language::Vi => "Sắp xếp",
            Language::En => "Sort",
        }
    }

    pub fn nav_demolish(&self) -> &'static str {
        match self {
            Language::Vi => "Phá dỡ",
            Language::En => "Demolish",
        }
    }

    pub fn nav_rescan(&self) -> &'static str {
        match self {
            Language::Vi => "Quét lại",
            Language::En => "Rescan",
        }
    }

    pub fn nav_lang(&self) -> &'static str {
        match self {
            Language::Vi => "Đổi ngôn ngữ",
            Language::En => "Language",
        }
    }

    pub fn nav_quit(&self) -> &'static str {
        match self {
            Language::Vi => "Thoát",
            Language::En => "Quit",
        }
    }

    // Navigation feedback messages
    pub fn drill_down_file_err(&self, name: &str) -> String {
        match self {
            Language::Vi => format!("'{}' là tệp đơn lẻ, không thể đi vào.", name),
            Language::En => format!("'{}' is a single file, cannot enter.", name),
        }
    }

    pub fn drill_down_empty_err(&self) -> &'static str {
        match self {
            Language::Vi => "Thư mục trống, không có công trình bên trong.",
            Language::En => "Empty directory, no buildings inside.",
        }
    }

    pub fn drill_down_entered(&self, name: &str) -> String {
        match self {
            Language::Vi => format!("Đã bước vào công trình: {}", name),
            Language::En => format!("Entered building: {}", name),
        }
    }

    pub fn navigate_up_success(&self) -> &'static str {
        match self {
            Language::Vi => "Đã lùi về con phố cha.",
            Language::En => "Returned to parent avenue.",
        }
    }

    pub fn zoning_switched(&self, mode_label: &str) -> String {
        match self {
            Language::Vi => format!("Đã quy hoạch lại: {}", mode_label),
            Language::En => format!("Zoning reorganized: {}", mode_label),
        }
    }

    pub fn open_success(&self, name: &str) -> String {
        match self {
            Language::Vi => format!("Đã mở '{}' bằng ứng dụng mặc định.", name),
            Language::En => format!("Opened '{}' with default app.", name),
        }
    }

    pub fn open_err(&self, name: &str) -> String {
        match self {
            Language::Vi => format!("Không thể mở '{}'.", name),
            Language::En => format!("Could not open '{}'.", name),
        }
    }

    // Demolition Modal
    pub fn demolish_modal_title(&self) -> &'static str {
        match self {
            Language::Vi => "[!] LỆNH PHÁ DỠ CÔNG TRÌNH [!]",
            Language::En => "[!] BUILDING DEMOLITION ORDER [!]",
        }
    }

    pub fn demolish_question(&self) -> &'static str {
        match self {
            Language::Vi => "Bạn có chắc chắn muốn giải tỏa công trình này?",
            Language::En => "Are you sure you want to demolish this building?",
        }
    }

    pub fn demolish_warning(&self) -> &'static str {
        match self {
            Language::Vi => "Hành động này sẽ XÓA VĨNH VIỄN dữ liệu trên đĩa!",
            Language::En => "This action will PERMANENTLY DELETE data from disk!",
        }
    }

    pub fn demolish_standard_prompt(&self) -> &'static str {
        match self {
            Language::Vi => "[ Y: Tiến hành phá dỡ | Esc: Hủy bỏ ]",
            Language::En => "[ Y: Proceed Demolition | Esc: Cancel ]",
        }
    }

    pub fn demolish_high_risk_notice(&self) -> &'static str {
        match self {
            Language::Vi => {
                "Cơ chế an toàn hai lớp: Đối tượng phá dỡ là Khu vực cấm hoặc có dung lượng >= 1GB."
            }
            Language::En => {
                "Two-Tier Safety Mechanism: Target is a Restricted Bunker or size >= 1GB."
            }
        }
    }

    pub fn demolish_high_risk_prompt(&self, typed: &str) -> String {
        match self {
            Language::Vi => format!("Nhập chính xác 'YES' để xác nhận giải tỏa: [{}]", typed),
            Language::En => format!("Type 'YES' exactly to confirm demolition: [{}]", typed),
        }
    }

    pub fn demolish_success(&self, path: &str) -> String {
        match self {
            Language::Vi => format!("Đã phá dỡ công trình: {}", path),
            Language::En => format!("Demolished building: {}", path),
        }
    }

    pub fn demolish_cancelled(&self) -> &'static str {
        match self {
            Language::Vi => "Đã hủy lệnh phá dỡ.",
            Language::En => "Demolition cancelled.",
        }
    }

    pub fn rescan_success(&self, count: usize) -> String {
        match self {
            Language::Vi => format!("Làm mới dữ liệu địa thành công! ({} công trình)", count),
            Language::En => format!("Terrain scan refreshed successfully! ({} buildings)", count),
        }
    }

    pub fn scrapyard_entered(&self) -> &'static str {
        match self {
            Language::Vi => {
                "Đã chuyển sang Đại Lộ Phế Liệu Hệ Thống. [D] Phá dỡ | [C] Dọn tất cả | [Esc] Quay lại"
            }
            Language::En => {
                "Switched to System Scrapyard Avenue. [D] Demolish | [C] Clean All | [Esc] Return"
            }
        }
    }

    pub fn scrapyard_exited(&self) -> &'static str {
        match self {
            Language::Vi => "Đã quay lại cây thư mục làm việc.",
            Language::En => "Returned to working filesystem.",
        }
    }

    pub fn bulldozer_success(&self, bytes_str: &str) -> String {
        match self {
            Language::Vi => {
                format!(
                    "Cỗ xe ủi đã dọn sạch bãi phế liệu: Giải phóng {}!",
                    bytes_str
                )
            }
            Language::En => {
                format!("Bulldozer swept the scrapyard: Reclaimed {}!", bytes_str)
            }
        }
    }

    // Building categories display
    pub fn category_name(&self, cat: crate::model::BuildingCategory, is_dir: bool) -> &'static str {
        if is_dir || cat == crate::model::BuildingCategory::Directory {
            return match self {
                Language::Vi => "Văn phòng (Thư mục)",
                Language::En => "Office (Directory)",
            };
        }
        match (self, cat) {
            (Language::Vi, crate::model::BuildingCategory::Code) => "Trung tâm Công nghệ (Code)",
            (Language::En, crate::model::BuildingCategory::Code) => "Tech Center (Code)",
            (Language::Vi, crate::model::BuildingCategory::Media) => "Khu Giải trí (Media)",
            (Language::En, crate::model::BuildingCategory::Media) => "Entertainment (Media)",
            (Language::Vi, crate::model::BuildingCategory::Archive) => "Kho Lưu trữ (Archive/Zip)",
            (Language::En, crate::model::BuildingCategory::Archive) => "Storage Vault (Archive)",
            (Language::Vi, crate::model::BuildingCategory::Binary) => {
                "Khu Công nghiệp (Binary/Lib)"
            }
            (Language::En, crate::model::BuildingCategory::Binary) => "Industrial Complex (Binary)",
            (Language::Vi, crate::model::BuildingCategory::Document) => {
                "Tòa soạn / Bưu điện (Tài liệu)"
            }
            (Language::En, crate::model::BuildingCategory::Document) => "Publishing House (Doc)",
            (Language::Vi, crate::model::BuildingCategory::Restricted) => "Khu Quân sự (Bunker)",
            (Language::En, crate::model::BuildingCategory::Restricted) => "Military Zone (Bunker)",
            (Language::Vi, crate::model::BuildingCategory::Junk) => "Bãi phế liệu (Rác hệ thống)",
            (Language::En, crate::model::BuildingCategory::Junk) => "Scrapyard (System Junk)",
            (Language::Vi, crate::model::BuildingCategory::Other)
            | (Language::Vi, crate::model::BuildingCategory::Directory) => "Dân cư (File khác)",
            (Language::En, crate::model::BuildingCategory::Other)
            | (Language::En, crate::model::BuildingCategory::Directory) => "Residential (Other)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_toggle() {
        assert_eq!(Language::Vi.toggle(), Language::En);
        assert_eq!(Language::En.toggle(), Language::Vi);
    }

    #[test]
    fn test_language_display() {
        assert_eq!(format!("{}", Language::Vi), "vi");
        assert_eq!(format!("{}", Language::En), "en");
        assert_eq!(Language::Vi.display_name(), "Tiếng Việt");
        assert_eq!(Language::En.display_name(), "English");
    }

    #[test]
    fn test_dictionary_strings_non_empty() {
        for lang in [Language::Vi, Language::En] {
            assert!(!lang.app_title().is_empty());
            assert!(!lang.wait_moment().is_empty());
            assert!(!lang.avenue_title("test").is_empty());
            assert!(!lang.bunker_tag().is_empty());
            assert!(!lang.protected_tag().is_empty());
            assert!(!lang.demolish_modal_title().is_empty());
            assert!(!lang.nav_stroll().is_empty());
            assert!(!lang.nav_lang().is_empty());
        }
    }

    #[test]
    fn test_bilingual_distinction() {
        assert_ne!(Language::Vi.app_title(), Language::En.app_title());
        assert_ne!(Language::Vi.nav_stroll(), Language::En.nav_stroll());
        assert_ne!(
            Language::Vi.demolish_modal_title(),
            Language::En.demolish_modal_title()
        );
        assert_ne!(Language::Vi.protected_tag(), Language::En.protected_tag());
    }

    #[test]
    fn test_navigation_feedback_bilingual() {
        assert_ne!(
            Language::Vi.drill_down_file_err("file.txt"),
            Language::En.drill_down_file_err("file.txt")
        );
        assert_ne!(
            Language::Vi.drill_down_empty_err(),
            Language::En.drill_down_empty_err()
        );
        assert_ne!(
            Language::Vi.drill_down_entered("folder"),
            Language::En.drill_down_entered("folder")
        );
        assert_ne!(
            Language::Vi.navigate_up_success(),
            Language::En.navigate_up_success()
        );
        assert_ne!(
            Language::Vi.zoning_switched("Mode"),
            Language::En.zoning_switched("Mode")
        );
    }
}
