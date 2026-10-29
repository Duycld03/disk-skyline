use super::draw::{building_category_badge, category_color, draw_text_f, fit_text_to_width};
use crate::gui::state::{BuildingPlacement2D, GuiApp};
use crate::model::{format_bytes, BuildingCategory, BuildingNode};
use macroquad::prelude::*;
use macroquad::text::Font;

pub fn render_street_radar(
    placements: &[BuildingPlacement2D],
    avatar_x: f32,
    max_street_x: f32,
    screen_w: f32,
    time: f64,
) {
    if placements.is_empty() || max_street_x <= screen_w {
        return;
    }

    let rx = 12.0;
    let ry = 70.0;
    let rw = screen_w - 24.0;
    let rh = 14.0;

    // Dark backdrop for radar
    draw_rectangle(rx, ry, rw, rh, Color::new(0.04, 0.06, 0.10, 0.90));
    draw_rectangle_lines(rx, ry, rw, rh, 1.0, Color::new(0.25, 0.35, 0.50, 0.70));

    // Draw building profile ticks
    for p in placements {
        let bx = rx + (p.x / max_street_x) * rw;
        let bw = ((p.width / max_street_x) * rw).clamp(4.0, 24.0);
        let bh = ((p.height / 380.0) * (rh - 4.0)).clamp(2.0, rh - 4.0);
        let by = ry + rh - 2.0 - bh;
        draw_rectangle(bx, by, bw, bh, Color::new(0.3, 0.6, 0.85, 0.8));
    }

    // Avatar position marker (blinking amber bar)
    let av_rx = (rx + (avatar_x / max_street_x) * rw).clamp(rx + 2.0, rx + rw - 4.0);
    let pulse = ((time * 6.0).sin() as f32 * 0.2 + 0.8).clamp(0.6, 1.0);
    draw_rectangle(
        av_rx - 2.0,
        ry + 2.0,
        4.0,
        rh - 4.0,
        Color::new(1.0, 0.85, 0.2, pulse),
    );
}

pub fn render_building(
    placement: &BuildingPlacement2D,
    node: &BuildingNode,
    is_selected: bool,
    screen_h: f32,
    camera_x: f32,
    time: f64,
    font: Option<&Font>,
) {
    let screen_x = placement.x - camera_x;
    let ground = GuiApp::ground_y(screen_h);
    let top_y = ground - placement.height;
    let cat = node.file_type;
    let col = category_color(cat);
    let is_stale = node.is_stale();

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
                if is_stale {
                    Color::new(0.55, 0.50, 0.30, 0.45) // Dim abandoned window
                } else {
                    Color::new(0.98, 0.90, 0.45, 0.85) // Warm lit window
                }
            } else {
                Color::new(0.08, 0.12, 0.16, 0.70) // Dark window
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
    if cat == BuildingCategory::Junk {
        // Scrapyard crane girder / hazard warning beacon
        let crane_pulse = ((time * 10.0).sin() as f32).abs();
        draw_line(
            roof_mid - 6.0,
            top_y,
            roof_mid + 6.0,
            top_y - 12.0,
            2.5,
            Color::new(0.85, 0.45, 0.18, 0.95),
        );
        draw_circle(
            roof_mid + 6.0,
            top_y - 12.0,
            3.0,
            Color::new(1.0, 0.65, 0.15, crane_pulse),
        );
    } else if node.is_dir {
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
        let lock_w = 56.0;
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
        draw_text_f("SYS LOCK", lock_x + 3.0, lock_y + 12.0, 11.0, WHITE, font);
    }

    // Upper Facade Neon Signboard (visible from afar)
    if placement.height >= 40.0 && placement.width >= 36.0 {
        let sign_y = top_y + if node.is_system_protected { 24.0 } else { 8.0 };
        let sign_h = 16.0;
        let sign_pad = 4.0;
        let sign_w = placement.width - sign_pad * 2.0;
        let sign_x = screen_x + sign_pad;

        draw_rectangle(
            sign_x,
            sign_y,
            sign_w,
            sign_h,
            Color::new(0.04, 0.05, 0.08, 0.90),
        );
        let border_c = if cat == BuildingCategory::Junk {
            let flick = ((time * 12.0).sin() as f32 * 0.15 + 0.85).clamp(0.6, 1.0);
            Color::new(0.95, 0.50, 0.15, flick)
        } else {
            Color::new(col.r, col.g, col.b, if is_selected { 0.95 } else { 0.65 })
        };
        draw_rectangle_lines(sign_x, sign_y, sign_w, sign_h, 1.0, border_c);

        let badge = if is_stale {
            "[OLD]"
        } else {
            building_category_badge(cat, node.is_dir, node.is_system_protected)
        };
        let full_text = format!("{} {}", badge, node.name);
        let fitted_text = fit_text_to_width(&full_text, sign_w - 6.0, 11.0, font);
        draw_text_f(
            &fitted_text,
            sign_x + 3.0,
            sign_y + 12.0,
            11.0,
            if is_selected {
                Color::new(1.0, 0.95, 0.4, 1.0)
            } else {
                col
            },
            font,
        );
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

    // Street-level Nameplate & Size along sidewalk curb
    {
        let plate_y = ground + 6.0;
        let plate_w = placement.width.max(48.0);
        let plate_x = screen_x + (placement.width - plate_w) / 2.0;
        let plate_h = 28.0;

        draw_rectangle(
            plate_x,
            plate_y,
            plate_w,
            plate_h,
            if is_selected {
                Color::new(0.12, 0.16, 0.24, 0.95)
            } else {
                Color::new(0.06, 0.08, 0.12, 0.85)
            },
        );
        draw_rectangle_lines(
            plate_x,
            plate_y,
            plate_w,
            plate_h,
            if is_selected { 1.5 } else { 1.0 },
            if is_selected {
                Color::new(1.0, 0.9, 0.2, 0.95)
            } else {
                Color::new(col.r * 0.7, col.g * 0.7, col.b * 0.7, 0.5)
            },
        );

        // Row 1: Name
        let name_text = fit_text_to_width(&node.name, plate_w - 6.0, 11.0, font);
        draw_text_f(
            &name_text,
            plate_x + 3.0,
            plate_y + 11.0,
            11.0,
            if is_selected {
                WHITE
            } else {
                Color::new(0.85, 0.88, 0.92, 1.0)
            },
            font,
        );

        // Row 2: Formatted Size
        let size_str = if is_stale {
            format!("{} [OLD]", format_bytes(node.size))
        } else {
            format_bytes(node.size)
        };
        let size_text = fit_text_to_width(&size_str, plate_w - 6.0, 10.0, font);
        draw_text_f(
            &size_text,
            plate_x + 3.0,
            plate_y + 23.0,
            10.0,
            Color::new(0.4, 0.85, 0.5, 0.95),
            font,
        );
    }
}
