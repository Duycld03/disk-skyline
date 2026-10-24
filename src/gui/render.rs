use crate::gui::state::{Avatar, BuildingPlacement2D, Direction, GuiApp, Particle, Star};
use crate::i18n::Language;
use crate::model::{format_bytes, BuildingCategory, BuildingNode};
use macroquad::prelude::*;

pub fn category_color(cat: BuildingCategory) -> Color {
    match cat {
        BuildingCategory::Directory => Color::new(0.25, 0.55, 0.95, 1.0), // Vibrant Blue
        BuildingCategory::Code => Color::new(0.20, 0.85, 0.40, 1.0),      // Neon Green
        BuildingCategory::Media => Color::new(0.92, 0.25, 0.75, 1.0),     // Cyberpunk Magenta
        BuildingCategory::Archive => Color::new(0.95, 0.75, 0.15, 1.0),   // Amber Industrial
        BuildingCategory::Binary => Color::new(0.95, 0.35, 0.20, 1.0),    // Molten Orange
        BuildingCategory::Document => Color::new(0.85, 0.88, 0.92, 1.0),  // Silver White
        BuildingCategory::Restricted => Color::new(0.90, 0.15, 0.15, 1.0), // Danger Red
        BuildingCategory::Other => Color::new(0.60, 0.65, 0.72, 1.0),     // Industrial Slate
    }
}

pub fn render_night_sky(stars: &[Star], screen_w: f32, screen_h: f32, camera_x: f32, time: f64) {
    // Sky gradient background
    clear_background(Color::new(0.04, 0.05, 0.10, 1.0)); // Deep Midnight Navy

    // Distant city silhouette glow on horizon
    let ground = GuiApp::ground_y(screen_h);
    draw_rectangle(
        0.0,
        ground - 140.0,
        screen_w,
        140.0,
        Color::new(0.08, 0.09, 0.18, 0.6),
    );

    // Parallax stars
    for star in stars {
        let parallax_x = star.x - camera_x * 0.15;
        let screen_x = parallax_x.rem_euclid(screen_w + 200.0) - 100.0;
        if screen_x >= -10.0 && screen_x <= screen_w + 10.0 && star.y < ground - 40.0 {
            let twinkle = ((time * star.twinkle_speed as f64).sin() as f32) * 0.3;
            let alpha = (star.base_alpha + twinkle).clamp(0.1, 1.0);
            draw_rectangle(
                screen_x,
                star.y,
                star.size,
                star.size,
                Color::new(1.0, 1.0, 1.0, alpha),
            );
        }
    }
}

pub fn render_street(screen_w: f32, screen_h: f32, camera_x: f32) {
    let ground = GuiApp::ground_y(screen_h);

    // Sidewalk curb
    draw_rectangle(
        0.0,
        ground,
        screen_w,
        12.0,
        Color::new(0.22, 0.24, 0.28, 1.0), // Curbstone gray
    );

    // Asphalt street
    draw_rectangle(
        0.0,
        ground + 12.0,
        screen_w,
        screen_h - (ground + 12.0),
        Color::new(0.12, 0.13, 0.15, 1.0), // Dark asphalt
    );

    // Street markings (dashed lines)
    let dash_w = 40.0;
    let dash_gap = 30.0;
    let step = dash_w + dash_gap;
    let start_offset = -(camera_x % step);
    let mut dx = start_offset;
    while dx < screen_w + step {
        draw_rectangle(
            dx,
            ground + 45.0,
            dash_w,
            4.0,
            Color::new(0.85, 0.75, 0.20, 0.7), // Yellow lane divider
        );
        dx += step;
    }

    // Streetlights along the avenue
    let light_step = 260.0;
    let light_offset = -(camera_x % light_step);
    let mut lx = light_offset;
    while lx < screen_w + light_step {
        // Lamp pole
        draw_rectangle(
            lx,
            ground - 80.0,
            4.0,
            80.0,
            Color::new(0.35, 0.38, 0.42, 1.0),
        );
        // Lamp head
        draw_rectangle(
            lx - 6.0,
            ground - 84.0,
            16.0,
            6.0,
            Color::new(0.95, 0.90, 0.50, 1.0),
        );
        // Light glow cone
        draw_triangle(
            Vec2::new(lx + 2.0, ground - 78.0),
            Vec2::new(lx - 45.0, ground),
            Vec2::new(lx + 49.0, ground),
            Color::new(1.0, 0.95, 0.60, 0.08),
        );
        lx += light_step;
    }
}

pub fn render_building(
    placement: &BuildingPlacement2D,
    node: &BuildingNode,
    is_selected: bool,
    screen_h: f32,
    camera_x: f32,
    time: f64,
) {
    let screen_x = placement.x - camera_x;
    let ground = GuiApp::ground_y(screen_h);
    let top_y = ground - placement.height;
    let cat = node.file_type;
    let col = category_color(cat);

    // Dark building core background
    draw_rectangle(
        screen_x,
        top_y,
        placement.width,
        placement.height,
        Color::new(col.r * 0.25, col.g * 0.25, col.b * 0.25, 0.95),
    );

    // Building facade outer frame
    let frame_thickness = if is_selected { 3.0 } else { 1.5 };
    let border_color = if is_selected {
        // Pulsing selection color
        let pulse = ((time * 6.0).sin() as f32 * 0.2 + 0.8).clamp(0.6, 1.0);
        Color::new(1.0, 0.95, 0.2, pulse)
    } else {
        col
    };
    draw_rectangle_lines(
        screen_x,
        top_y,
        placement.width,
        placement.height,
        frame_thickness,
        border_color,
    );

    // Windows Grid inside facade
    let win_size = 6.0;
    let win_gap = 10.0;
    let start_win_y = top_y + 12.0;
    let end_win_y = ground - 24.0;
    let start_win_x = screen_x + 8.0;
    let end_win_x = screen_x + placement.width - 8.0;

    let mut wy = start_win_y;
    let mut row_idx = 0;
    while wy + win_size <= end_win_y {
        let mut wx = start_win_x;
        let mut col_idx = 0;
        while wx + win_size <= end_win_x {
            // Pseudo-random light pattern based on coordinates
            let seed = (placement.node_index * 31 + row_idx * 17 + col_idx * 7) % 10;
            let is_lit = seed < 6;
            let win_col = if is_lit {
                Color::new(0.98, 0.90, 0.45, 0.85) // Warm lit window
            } else {
                Color::new(0.10, 0.15, 0.20, 0.70) // Dark window
            };
            draw_rectangle(wx, wy, win_size, win_size, win_col);
            wx += win_size + win_gap;
            col_idx += 1;
        }
        wy += win_size + win_gap;
        row_idx += 1;
    }

    // Ground entrance doorway
    let door_w = (placement.width * 0.26).clamp(18.0, 32.0);
    let door_h = 22.0;
    let door_x = screen_x + (placement.width - door_w) / 2.0;
    let door_y = ground - door_h;

    // Doorway opening
    draw_rectangle(
        door_x,
        door_y,
        door_w,
        door_h,
        Color::new(0.08, 0.08, 0.12, 1.0),
    );
    draw_rectangle_lines(
        door_x,
        door_y,
        door_w,
        door_h,
        1.5,
        Color::new(0.9, 0.75, 0.3, 0.9),
    );

    // Rooftop Antenna / Accent
    let roof_mid = screen_x + placement.width / 2.0;
    if node.is_dir {
        // Heliport / Signal beacon on directory towers
        draw_line(
            roof_mid,
            top_y,
            roof_mid,
            top_y - 14.0,
            2.0,
            Color::new(0.7, 0.7, 0.8, 1.0),
        );
        let beacon_pulse = ((time * 8.0).sin() as f32).max(0.0);
        draw_circle(
            roof_mid,
            top_y - 14.0,
            3.0,
            Color::new(1.0, 0.2, 0.2, beacon_pulse),
        );
    }

    // System protected badge
    if node.is_system_protected {
        let lock_w = 48.0;
        let lock_h = 16.0;
        let lock_x = screen_x + (placement.width - lock_w) / 2.0;
        let lock_y = top_y + 4.0;
        draw_rectangle(
            lock_x,
            lock_y,
            lock_w,
            lock_h,
            Color::new(0.85, 0.15, 0.15, 0.9),
        );
        draw_text("SYS LOCK", lock_x + 3.0, lock_y + 12.0, 12.0, WHITE);
    }

    // Animated selection indicator arrow
    if is_selected {
        let arrow_bounce = ((time * 7.0).sin() as f32) * 4.0;
        let arrow_tip_y = top_y - 8.0 + arrow_bounce;
        draw_triangle(
            Vec2::new(roof_mid - 7.0, arrow_tip_y - 12.0),
            Vec2::new(roof_mid + 7.0, arrow_tip_y - 12.0),
            Vec2::new(roof_mid, arrow_tip_y),
            Color::new(1.0, 0.9, 0.1, 1.0),
        );
    }
}

pub fn render_avatar(avatar: &Avatar, screen_h: f32, camera_x: f32) {
    let screen_x = avatar.x - camera_x;
    let screen_y = avatar.y;

    // Stride leg offset when walking
    let leg_offset = if avatar.is_grounded && avatar.walk_timer > 0.0 {
        (avatar.walk_timer.sin() * 4.0).round()
    } else if !avatar.is_grounded {
        -3.0 // Jumping tuck
    } else {
        0.0
    };

    // Shadow on street
    let ground = GuiApp::ground_y(screen_h);
    let shadow_scale =
        (1.0 - (ground - (screen_y + GuiApp::AVATAR_HEIGHT)) / 100.0).clamp(0.4, 1.0);
    draw_ellipse(
        screen_x + GuiApp::AVATAR_WIDTH / 2.0,
        ground - 1.0,
        14.0 * shadow_scale,
        4.0 * shadow_scale,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.45),
    );

    // 1. Head (14x14 px)
    let head_x = screen_x + (GuiApp::AVATAR_WIDTH - 14.0) / 2.0;
    let head_y = screen_y;
    // Hair
    draw_rectangle(head_x, head_y, 14.0, 5.0, Color::new(0.18, 0.12, 0.08, 1.0));
    // Face skin
    draw_rectangle(
        head_x,
        head_y + 4.0,
        14.0,
        10.0,
        Color::new(0.98, 0.82, 0.72, 1.0),
    );

    // Pixel Eyes
    let (eye1_x, eye2_x) = if avatar.facing == Direction::Right {
        (head_x + 6.0, head_x + 10.0)
    } else {
        (head_x + 2.0, head_x + 6.0)
    };
    draw_rectangle(
        eye1_x,
        head_y + 7.0,
        2.0,
        3.0,
        Color::new(0.08, 0.08, 0.10, 1.0),
    );
    draw_rectangle(
        eye2_x,
        head_y + 7.0,
        2.0,
        3.0,
        Color::new(0.08, 0.08, 0.10, 1.0),
    );

    // 2. Torso / Cyberpunk Hoodie (18x14 px)
    let body_x = screen_x + (GuiApp::AVATAR_WIDTH - 18.0) / 2.0;
    let body_y = screen_y + 14.0;
    draw_rectangle(
        body_x,
        body_y,
        18.0,
        14.0,
        Color::new(0.12, 0.65, 0.85, 1.0), // Neon Teal / Cyan Hoodie
    );
    // Vest zipper line
    draw_rectangle(
        body_x + 8.0,
        body_y,
        2.0,
        14.0,
        Color::new(0.95, 0.45, 0.15, 1.0), // Orange stripe
    );

    // 3. Legs & Boots
    let leg_w = 5.0;
    let leg_h = 10.0;
    let left_leg_x = screen_x + 3.0;
    let right_leg_x = screen_x + GuiApp::AVATAR_WIDTH - 3.0 - leg_w;
    let leg_y = screen_y + 28.0;

    // Left leg
    draw_rectangle(
        left_leg_x,
        leg_y + leg_offset,
        leg_w,
        leg_h,
        Color::new(0.15, 0.16, 0.20, 1.0),
    );
    // Right leg
    draw_rectangle(
        right_leg_x,
        leg_y - leg_offset,
        leg_w,
        leg_h,
        Color::new(0.15, 0.16, 0.20, 1.0),
    );
}

pub fn render_particles(particles: &[Particle], camera_x: f32) {
    for p in particles {
        let sx = p.x - camera_x;
        let alpha = (1.0 - (p.life / p.max_life)).clamp(0.0, 1.0);
        let col = Color::new(p.color.r, p.color.g, p.color.b, alpha);
        draw_rectangle(sx, p.y, p.size, p.size, col);
    }
}

pub fn render_rpg_hud(
    app: &crate::app::App,
    selected_node: Option<&BuildingNode>,
    screen_w: f32,
    screen_h: f32,
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
    draw_text(title, 24.0, 30.0, 18.0, Color::new(0.3, 0.9, 0.95, 1.0));

    let trail = app.breadcrumbs().join(" / ");
    let avenue_str = lang.avenue_title(&trail);
    draw_text(
        &avenue_str,
        24.0,
        52.0,
        15.0,
        Color::new(0.85, 0.85, 0.85, 1.0),
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
    draw_text(
        &capacity_label,
        bar_x,
        bar_y - 4.0,
        13.0,
        Color::new(0.8, 0.8, 0.8, 1.0),
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
        "[L] Đổi sang English".to_string()
    } else {
        "[L] Switch to Tiếng Việt".to_string()
    };
    draw_text(
        &lang_hint,
        bar_x,
        bar_y + 32.0,
        13.0,
        Color::new(1.0, 0.85, 0.25, 1.0),
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
        draw_text(&name_str, 24.0, card_y + 26.0, 18.0, WHITE);

        // Category Badge with dynamic width
        let cat_label = cat.display_label_lang(lang);
        let badge_dim = measure_text(cat_label, None, 13, 1.0);
        let badge_w = (badge_dim.width + 18.0).max(120.0);
        draw_rectangle(24.0, card_y + 35.0, badge_w, 20.0, cat_col);
        draw_text(
            cat_label,
            32.0,
            card_y + 49.0,
            13.0,
            Color::new(0.05, 0.05, 0.05, 1.0),
        );

        // Size & Items info dynamically positioned after badge
        let size_str = if lang == Language::Vi {
            format!("Dung lượng: {}", format_bytes(node.size))
        } else {
            format!("Size: {}", format_bytes(node.size))
        };
        let size_x = 24.0 + badge_w + 18.0;
        draw_text(
            &size_str,
            size_x,
            card_y + 50.0,
            15.0,
            Color::new(0.9, 0.9, 0.9, 1.0),
        );

        let size_dim = measure_text(&size_str, None, 15, 1.0);
        let next_x = size_x + size_dim.width + 24.0;

        if node.is_dir {
            let child_str = if lang == Language::Vi {
                format!("Quy mô: {} công trình con", node.children.len())
            } else {
                format!("Scale: {} sub-buildings", node.children.len())
            };
            draw_text(
                &child_str,
                next_x,
                card_y + 50.0,
                15.0,
                Color::new(0.7, 0.85, 1.0, 1.0),
            );
        }

        if node.is_system_protected {
            let lock_notice = if lang == Language::Vi {
                "[KHÓA BẢO VỆ HỆ THỐNG - KHÔNG THỂ PHÁ DỠ]"
            } else {
                "[SYSTEM PROTECTED LOCK - DEMOLITION FORBIDDEN]"
            };
            draw_text(
                lock_notice,
                next_x + 220.0,
                card_y + 50.0,
                14.0,
                Color::new(1.0, 0.3, 0.3, 1.0),
            );
        }
    } else {
        draw_text(
            "Đang đứng trên đường trống...",
            24.0,
            card_y + 36.0,
            16.0,
            GRAY,
        );
    }

    // Key command bar hints
    let hotkeys = if lang == Language::Vi {
        "[A/D/←/→] Đi bộ  [Space] Nhảy  [Enter] Vào  [Esc/Backspace] Lùi  [Tab] Sắp xếp  [D] Phá dỡ  [F11] Toàn màn hình  [Q] Thoát"
    } else {
        "[A/D/←/→] Walk  [Space] Jump  [Enter] Enter  [Esc/Backspace] Back  [Tab] Sort  [D] Demolish  [F11] Fullscreen  [Q] Quit"
    };
    draw_text(
        hotkeys,
        24.0,
        card_y + 85.0,
        14.0,
        Color::new(1.0, 0.85, 0.3, 0.95),
    );
}

pub fn render_demolish_modal(
    modal: &crate::gui::state::GuiDemolishModal,
    screen_w: f32,
    screen_h: f32,
) {
    // Dim background
    draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.7));

    let mw = 520.0;
    let mh = 200.0;
    let mx = (screen_w - mw) / 2.0;
    let my = (screen_h - mh) / 2.0;

    // Modal background
    draw_rectangle(mx, my, mw, mh, Color::new(0.12, 0.08, 0.08, 0.98));
    draw_rectangle_lines(mx, my, mw, mh, 2.5, Color::new(0.9, 0.2, 0.2, 1.0));

    // Title
    draw_text(
        "LỆNH PHÁ DỠ CÔNG TRÌNH (DEMOLITION ORDER)",
        mx + 25.0,
        my + 36.0,
        18.0,
        Color::new(1.0, 0.3, 0.3, 1.0),
    );

    let info = format!(
        "Bạn có chắc muốn san phẳng: {} ({})?",
        modal.target_name, modal.target_size_formatted
    );
    draw_text(&info, mx + 25.0, my + 80.0, 16.0, WHITE);

    let warn = "Hành động này sẽ XÓA VĨNH VIỄN tệp/thư mục khỏi đĩa lưu trữ!";
    draw_text(
        warn,
        mx + 25.0,
        my + 112.0,
        13.0,
        Color::new(0.9, 0.7, 0.2, 1.0),
    );

    // Buttons
    draw_rectangle(
        mx + 40.0,
        my + 140.0,
        180.0,
        34.0,
        Color::new(0.85, 0.15, 0.15, 1.0),
    );
    draw_text("[Y] Xác nhận phá dỡ", mx + 55.0, my + 162.0, 15.0, WHITE);

    draw_rectangle(
        mx + 280.0,
        my + 140.0,
        180.0,
        34.0,
        Color::new(0.25, 0.28, 0.35, 1.0),
    );
    draw_text("[N/Esc] Hủy bỏ", mx + 325.0, my + 162.0, 15.0, WHITE);
}

pub fn render_banner(msg: &str, is_error: bool, screen_w: f32) {
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
    draw_text(msg, bx + 16.0, by + 25.0, 15.0, WHITE);
}
