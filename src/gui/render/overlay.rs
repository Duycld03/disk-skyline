use super::draw::{category_color, draw_text_f, measure_text_f};
use crate::app::App;
use crate::gui::state::GuiDemolishModal;
use crate::i18n::Language;
use crate::model::{format_bytes, BuildingNode};
use macroquad::prelude::*;
use macroquad::text::Font;

pub fn render_rpg_hud(
    app: &App,
    selected_node: Option<&BuildingNode>,
    screen_w: f32,
    screen_h: f32,
    font: Option<&Font>,
) {
    let lang = app.language;

    // --- TOP BAR: Header & Stats ---
    draw_rectangle(
        12.0,
        10.0,
        screen_w - 24.0,
        56.0,
        Color::new(0.06, 0.08, 0.12, 0.92),
    );
    draw_rectangle_lines(
        12.0,
        10.0,
        screen_w - 24.0,
        56.0,
        1.5,
        Color::new(0.3, 0.5, 0.7, 0.8),
    );

    // Game Title & Avenue
    let title = "DISK SKYLINE 2D [MACROQUAD]";
    draw_text_f(
        title,
        24.0,
        30.0,
        18.0,
        Color::new(0.3, 0.9, 0.95, 1.0),
        font,
    );

    let trail = app.breadcrumbs().join(" / ");
    let avenue_str = lang.avenue_title(&trail);
    draw_text_f(
        &avenue_str,
        24.0,
        52.0,
        15.0,
        Color::new(0.85, 0.85, 0.85, 1.0),
        font,
    );

    // Disk Free Capacity Bar
    let bar_w = 260.0;
    let bar_h = 16.0;
    let bar_x = screen_w - bar_w - 36.0;
    let bar_y = 20.0;

    let root_size = app.root_node.size;
    let capacity_label = if lang == Language::Vi {
        format!("Tổng dung lượng: {}", format_bytes(root_size))
    } else {
        format!("Total Capacity: {}", format_bytes(root_size))
    };
    draw_text_f(
        &capacity_label,
        bar_x,
        bar_y - 4.0,
        13.0,
        Color::new(0.8, 0.8, 0.8, 1.0),
        font,
    );
    // Outer bar border
    draw_rectangle(
        bar_x,
        bar_y,
        bar_w,
        bar_h,
        Color::new(0.12, 0.14, 0.18, 1.0),
    );
    draw_rectangle(
        bar_x + 2.0,
        bar_y + 2.0,
        (bar_w - 4.0) * 0.72,
        bar_h - 4.0,
        Color::new(0.2, 0.8, 0.45, 0.9), // Green health bar
    );
    draw_rectangle_lines(
        bar_x,
        bar_y,
        bar_w,
        bar_h,
        1.2,
        Color::new(0.4, 0.6, 0.8, 1.0),
    );

    let lang_hint = if lang == Language::Vi {
        "[o] Mở  [/] Tìm  [L] English".to_string()
    } else {
        "[o] Open  [/] Search  [L] Tiếng Việt".to_string()
    };
    draw_text_f(
        &lang_hint,
        bar_x,
        bar_y + 32.0,
        13.0,
        Color::new(1.0, 0.85, 0.25, 1.0),
        font,
    );

    // --- BOTTOM BAR: RPG Target Card & Hotkeys ---
    let card_h = 100.0;
    let card_y = screen_h - card_h - 12.0;

    draw_rectangle(
        12.0,
        card_y,
        screen_w - 24.0,
        card_h,
        Color::new(0.06, 0.08, 0.12, 0.94),
    );
    draw_rectangle_lines(
        12.0,
        card_y,
        screen_w - 24.0,
        card_h,
        1.5,
        Color::new(0.3, 0.5, 0.7, 0.8),
    );

    if let Some(node) = selected_node {
        let cat = node.file_type;
        let cat_col = category_color(cat);

        // Building Name & Badge
        let name_prefix = if lang == Language::Vi {
            "CÔNG TRÌNH: "
        } else {
            "BUILDING: "
        };
        let name_str = format!("{}{}", name_prefix, node.name);
        draw_text_f(&name_str, 24.0, card_y + 26.0, 18.0, WHITE, font);

        // Category Badge with dynamic width
        let cat_label = cat.display_label_lang(lang);
        let badge_width = measure_text_f(cat_label, 13.0, font);
        let badge_w = (badge_width + 18.0).max(120.0);
        draw_rectangle(24.0, card_y + 35.0, badge_w, 20.0, cat_col);
        draw_text_f(
            cat_label,
            32.0,
            card_y + 49.0,
            13.0,
            Color::new(0.05, 0.05, 0.05, 1.0),
            font,
        );

        // Size & Items info dynamically positioned after badge
        let size_str = if lang == Language::Vi {
            format!("Dung lượng: {}", format_bytes(node.size))
        } else {
            format!("Size: {}", format_bytes(node.size))
        };
        let size_x = 24.0 + badge_w + 18.0;
        draw_text_f(
            &size_str,
            size_x,
            card_y + 50.0,
            15.0,
            Color::new(0.9, 0.9, 0.9, 1.0),
            font,
        );

        let size_width = measure_text_f(&size_str, 15.0, font);
        let next_x = size_x + size_width + 24.0;

        if node.is_dir {
            let child_str = if lang == Language::Vi {
                format!("Quy mô: {} công trình con", node.children.len())
            } else {
                format!("Scale: {} sub-buildings", node.children.len())
            };
            draw_text_f(
                &child_str,
                next_x,
                card_y + 50.0,
                15.0,
                Color::new(0.7, 0.85, 1.0, 1.0),
                font,
            );
        }

        if node.is_system_protected {
            let lock_notice = if app.is_in_scrapyard() {
                if node.name.to_lowercase().contains("snap") {
                    if lang == Language::Vi {
                        "[YÊU CẦU SUDO - Chạy trong terminal: sudo snap remove <pkg> --revision <rev>]"
                    } else {
                        "[SUDO REQUIRED - Run in terminal: sudo snap remove <pkg> --revision <rev>]"
                    }
                } else if node.name.to_lowercase().contains("apt") {
                    if lang == Language::Vi {
                        "[YÊU CẦU SUDO - Chạy trong terminal: sudo apt-get clean]"
                    } else {
                        "[SUDO REQUIRED - Run in terminal: sudo apt-get clean]"
                    }
                } else if lang == Language::Vi {
                    "[YÊU CẦU SUDO - Khóa bảo vệ hệ thống]"
                } else {
                    "[SUDO REQUIRED - System Protected Lock]"
                }
            } else if lang == Language::Vi {
                "[KHÓA BẢO VỆ HỆ THỐNG - KHÔNG THỂ PHÁ DỠ]"
            } else {
                "[SYSTEM PROTECTED LOCK - DEMOLITION FORBIDDEN]"
            };
            draw_text_f(
                lock_notice,
                next_x + 220.0,
                card_y + 50.0,
                14.0,
                Color::new(1.0, 0.3, 0.3, 1.0),
                font,
            );
        }
    } else {
        let empty_msg = if lang == Language::Vi {
            "Đang đứng trên đường trống..."
        } else {
            "Standing on open street..."
        };
        draw_text_f(empty_msg, 24.0, card_y + 36.0, 16.0, GRAY, font);
    }

    // Key command bar hints
    let hotkeys = if app.is_in_scrapyard() {
        if lang == Language::Vi {
            "[A/D] Đi bộ  [W/Space] Nhảy  [X] Phá dỡ phế liệu  [C] Xe ủi Dọn Tất Cả  [Esc] Quay lại phố"
        } else {
            "[A/D] Walk  [W/Space] Jump  [X] Demolish Scrap  [C] Bulldozer Clean All  [Esc] Back to Files"
        }
    } else if lang == Language::Vi {
        "[A/D/Left/Right] Đi bộ  [Space] Nhảy  [Enter] Vào  [o] Mở  [/] Tìm  [J] Phế liệu  [Esc] Lùi  [Tab] Sắp xếp  [X] Phá dỡ  [F11] Fullscreen  [Q] Thoát"
    } else {
        "[A/D/Left/Right] Walk  [Space] Jump  [Enter] Enter  [o] Open  [/] Search  [J] Scrapyard  [Esc] Back  [Tab] Sort  [X] Demolish  [F11] Fullscreen  [Q] Quit"
    };
    draw_text_f(
        hotkeys,
        24.0,
        card_y + 85.0,
        14.0,
        Color::new(1.0, 0.85, 0.3, 0.95),
        font,
    );
}

pub fn render_demolish_modal(
    modal: &GuiDemolishModal,
    lang: Language,
    screen_w: f32,
    screen_h: f32,
    font: Option<&Font>,
) {
    // Dim background
    draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.7));

    let mw = 540.0;
    let mh = 200.0;
    let mx = (screen_w - mw) / 2.0;
    let my = (screen_h - mh) / 2.0;

    // Modal background
    draw_rectangle(mx, my, mw, mh, Color::new(0.12, 0.08, 0.08, 0.98));
    draw_rectangle_lines(mx, my, mw, mh, 2.5, Color::new(0.9, 0.2, 0.2, 1.0));

    // Title
    let title = lang.demolish_modal_title();
    draw_text_f(
        title,
        mx + 25.0,
        my + 36.0,
        18.0,
        Color::new(1.0, 0.3, 0.3, 1.0),
        font,
    );

    let info = if lang == Language::Vi {
        format!(
            "Bạn có chắc muốn san phẳng: {} ({})?",
            modal.target_name, modal.target_size_formatted
        )
    } else {
        format!(
            "Are you sure you want to demolish: {} ({})?",
            modal.target_name, modal.target_size_formatted
        )
    };
    draw_text_f(&info, mx + 25.0, my + 80.0, 16.0, WHITE, font);

    let warn = lang.demolish_warning();
    draw_text_f(
        warn,
        mx + 25.0,
        my + 112.0,
        13.0,
        Color::new(0.9, 0.7, 0.2, 1.0),
        font,
    );

    // Buttons
    draw_rectangle(
        mx + 40.0,
        my + 140.0,
        200.0,
        34.0,
        Color::new(0.85, 0.15, 0.15, 1.0),
    );
    let (confirm_btn, cancel_btn) = if lang == Language::Vi {
        ("[Y] Tiến hành phá dỡ", "[N/Esc] Hủy bỏ")
    } else {
        ("[Y] Confirm Demolition", "[N/Esc] Cancel")
    };
    draw_text_f(confirm_btn, mx + 50.0, my + 162.0, 15.0, WHITE, font);

    draw_rectangle(
        mx + 300.0,
        my + 140.0,
        180.0,
        34.0,
        Color::new(0.25, 0.28, 0.35, 1.0),
    );
    draw_text_f(cancel_btn, mx + 345.0, my + 162.0, 15.0, WHITE, font);
}

pub fn render_search_overlay(
    query: &str,
    matched_node: Option<&BuildingNode>,
    lang: Language,
    screen_w: f32,
    font: Option<&Font>,
) {
    let bw = 540.0;
    let bh = 50.0;
    let bx = (screen_w - bw) / 2.0;
    let by = 90.0;

    // Background card
    draw_rectangle(bx, by, bw, bh, Color::new(0.06, 0.08, 0.14, 0.96));
    draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::new(0.2, 0.85, 0.95, 1.0));

    let (title_label, hint_label) = if lang == Language::Vi {
        (
            "TÌM KIẾM:",
            "[Enter] Giữ vị trí  [Esc] Đóng  (Gõ tên công trình)",
        )
    } else {
        (
            "SEARCH:",
            "[Enter] Confirm  [Esc] Close  (Type building name)",
        )
    };

    let display_query = if query.len() > 30 {
        format!("...{}_", &query[query.len() - 27..])
    } else {
        format!("{}_", query)
    };
    let search_text = format!(">> {} {}", title_label, display_query);
    draw_text_f(
        &search_text,
        bx + 16.0,
        by + 20.0,
        15.0,
        Color::new(1.0, 0.9, 0.2, 1.0),
        font,
    );

    let trimmed = query.trim();
    if trimmed.is_empty() {
        draw_text_f(
            hint_label,
            bx + 16.0,
            by + 38.0,
            11.0,
            Color::new(0.65, 0.70, 0.78, 1.0),
            font,
        );
    } else {
        let is_match = matched_node
            .map(|m| m.name.to_lowercase().contains(&trimmed.to_lowercase()))
            .unwrap_or(false);

        if is_match {
            let matched = matched_node.unwrap();
            let match_str = format!("→ {} ({})", matched.name, matched.formatted_size());
            draw_text_f(
                &match_str,
                bx + 16.0,
                by + 38.0,
                12.0,
                Color::new(0.3, 0.95, 0.5, 1.0),
                font,
            );
        } else {
            let no_match_msg = if lang == Language::Vi {
                format!("(x) Không tìm thấy công trình nào khớp với '{}'", trimmed)
            } else {
                format!("(x) No building matches '{}'", trimmed)
            };
            draw_text_f(
                &no_match_msg,
                bx + 16.0,
                by + 38.0,
                11.0,
                Color::new(1.0, 0.45, 0.35, 1.0),
                font,
            );
        }
    }
}

pub fn render_banner(msg: &str, is_error: bool, screen_w: f32, font: Option<&Font>) {
    let bw = 640.0;
    let bh = 40.0;
    let bx = (screen_w - bw) / 2.0;
    let by = 74.0;

    let bg = if is_error {
        Color::new(0.8, 0.1, 0.1, 0.95)
    } else {
        Color::new(0.1, 0.65, 0.3, 0.95)
    };

    draw_rectangle(bx, by, bw, bh, bg);
    draw_rectangle_lines(bx, by, bw, bh, 1.5, WHITE);
    draw_text_f(msg, bx + 16.0, by + 25.0, 15.0, WHITE, font);
}
