use crate::gui::state::{Avatar, Direction, GuiApp, Particle, Star};
use macroquad::prelude::*;

pub fn render_night_sky(
    stars: &[Star],
    screen_w: f32,
    screen_h: f32,
    camera_x: f32,
    time: f64,
    is_scrapyard: bool,
) {
    // Sky gradient background
    if is_scrapyard {
        clear_background(Color::new(0.08, 0.05, 0.07, 1.0)); // Industrial Smog Scrapyard
    } else {
        clear_background(Color::new(0.04, 0.05, 0.10, 1.0)); // Deep Midnight Navy
    }
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
