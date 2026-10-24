pub mod render;
pub mod state;

use crate::app::App;
use crate::demolish::execute_demolition;
use crate::gui::render::*;
use crate::gui::state::{Direction, GuiApp, GuiDemolishModal};
use crate::i18n::Language;
use crate::model::format_bytes;
use macroquad::prelude::*;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Disk Skyline 2D - Macroquad Engine".to_string(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

const FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/font.ttf");

pub async fn run_gui(app: App) {
    let mut gui = GuiApp::new(app);
    if let Ok(loaded_font) = macroquad::text::load_ttf_font_from_bytes(FONT_DATA) {
        gui.font = Some(loaded_font);
    }
    loop {
        let dt = get_frame_time().min(0.1);
        let screen_w = screen_width();
        let screen_h = screen_height();
        let time = get_time();

        // 1. Calculate building placements for current directory
        let placements = gui.compute_placements(screen_h);
        let max_street_x = placements
            .last()
            .map(|p| p.x + p.width + GuiApp::STREET_MARGIN_LEFT)
            .unwrap_or(screen_w);

        // 2. Handle Inputs
        if let Some(modal) = gui.modal.clone() {
            // Modal input handling
            if is_key_pressed(KeyCode::Y) {
                let target_idx = modal.target_index;
                let target_name = modal.target_name.clone();
                let current_lang = gui.app.language;

                // Find placement to spawn explosion particles
                if let Some(p) = placements.iter().find(|pl| pl.node_index == target_idx) {
                    let col = {
                        let curr = gui.app.current_node();
                        curr.children
                            .get(target_idx)
                            .map(|c| category_color(c.file_type))
                            .unwrap_or(WHITE)
                    };
                    let ground = GuiApp::ground_y(screen_h);
                    let top_y = ground - p.height;
                    gui.spawn_demolition_particles(p.x, top_y, p.width, p.height, col);
                }

                // Execute demolition logic
                let demolish_state = {
                    let curr = gui.app.current_node();
                    curr.children
                        .get(target_idx)
                        .map(crate::demolish::DemolishState::new)
                };

                if let Some(state) = demolish_state {
                    let curr = gui.app.current_node_mut();
                    match execute_demolition(&state, curr, target_idx) {
                        Ok(freed) => {
                            let msg = if current_lang == Language::Vi {
                                format!(
                                    "Đã phá dỡ thành công '{}', giải phóng {}!",
                                    target_name,
                                    format_bytes(freed)
                                )
                            } else {
                                format!(
                                    "Successfully demolished '{}', freed {}!",
                                    target_name,
                                    format_bytes(freed)
                                )
                            };
                            gui.banner_message = Some((msg, false, 3.5));
                        }
                        Err(e) => {
                            let err_msg = format!("Lỗi khi phá dỡ: {}", e);
                            gui.banner_message = Some((err_msg, true, 3.5));
                        }
                    }
                }

                gui.modal = None;
            } else if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                let cancel_msg = if gui.app.language == Language::Vi {
                    "Đã hủy bỏ lệnh phá dỡ.".to_string()
                } else {
                    "Demolition cancelled.".to_string()
                };
                gui.banner_message = Some((cancel_msg, false, 2.0));
                gui.modal = None;
            }
        } else if gui.app.search_query.is_some() {
            let prev_idx = gui.app.nav.selected_index;
            if is_key_pressed(KeyCode::Escape) {
                gui.app.cancel_search();
                gui.backspace_timer = 0.0;
            } else if is_key_pressed(KeyCode::Enter) {
                gui.app.confirm_search();
                gui.backspace_timer = 0.0;
            } else {
                // Continuous Backspace key-repeat
                if is_key_down(KeyCode::Backspace) {
                    if is_key_pressed(KeyCode::Backspace) {
                        gui.app.backspace_search();
                        gui.backspace_timer = 0.0;
                    } else {
                        gui.backspace_timer += dt;
                        if gui.backspace_timer >= 0.35 {
                            gui.app.backspace_search();
                            gui.backspace_timer = 0.31;
                        }
                    }
                } else {
                    gui.backspace_timer = 0.0;
                }

                // Poll typed characters
                while let Some(ch) = get_char_pressed() {
                    if !ch.is_control() {
                        gui.app.update_search(ch);
                    }
                }
            }

            // If selected building changed during search, warp avatar smoothly to target
            if gui.app.nav.selected_index != prev_idx {
                if let Some(p) = placements
                    .iter()
                    .find(|pl| pl.node_index == gui.app.nav.selected_index)
                {
                    gui.avatar.x = p.x + p.width / 2.0;
                    gui.avatar.vx = 0.0;
                }
            }
        } else {
            // Flush character buffer continuously during exploration so movement keys don't leak into search
            while get_char_pressed().is_some() {}
            gui.backspace_timer = 0.0;
            let mut move_dir: f32 = 0.0;

            if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                move_dir -= 1.0;
                gui.avatar.facing = Direction::Left;
            }
            if is_key_down(KeyCode::Right)
                || (is_key_down(KeyCode::D)
                    && !is_key_down(KeyCode::LeftControl)
                    && !is_key_down(KeyCode::LeftShift))
            {
                move_dir += 1.0;
                gui.avatar.facing = Direction::Right;
            }

            gui.avatar.vx = move_dir * GuiApp::WALK_SPEED;

            // Jumping (Space or W or Up arrow)
            if (is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::W)
                || is_key_pressed(KeyCode::Up))
                && gui.avatar.is_grounded
            {
                gui.avatar.vy = GuiApp::JUMP_VELOCITY;
                gui.avatar.is_grounded = false;
            }

            // Drill-down into directory
            if is_key_pressed(KeyCode::Enter) {
                if gui.app.drill_down() {
                    gui.avatar.x = GuiApp::STREET_MARGIN_LEFT;
                    gui.avatar.vx = 0.0;
                    gui.avatar.vy = 0.0;
                    gui.camera_x = 0.0;
                    gui.particles.clear();
                    if let Some((msg, is_err)) = gui.app.status_message.take() {
                        gui.banner_message = Some((msg, is_err, 2.0));
                    }
                } else if let Some((msg, is_err)) = gui.app.status_message.take() {
                    gui.banner_message = Some((msg, is_err, 2.5));
                }
            }

            // Navigate up
            if (is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::Escape))
                && gui.app.navigate_up()
            {
                gui.avatar.x = GuiApp::STREET_MARGIN_LEFT;
                gui.avatar.vx = 0.0;
                gui.avatar.vy = 0.0;
                gui.camera_x = 0.0;
                gui.particles.clear();
                if let Some((msg, is_err)) = gui.app.status_message.take() {
                    gui.banner_message = Some((msg, is_err, 2.0));
                }
            }

            // Toggle Sort
            if is_key_pressed(KeyCode::Tab) {
                gui.app.toggle_sort();
                if let Some((msg, is_err)) = gui.app.status_message.take() {
                    gui.banner_message = Some((msg, is_err, 2.0));
                }
            }

            // Toggle Language
            if is_key_pressed(KeyCode::L) {
                gui.app.language = gui.app.language.toggle();
                let lang_msg = if gui.app.language == Language::Vi {
                    "Đã chuyển ngôn ngữ sang Tiếng Việt.".to_string()
                } else {
                    "Language switched to English.".to_string()
                };
                gui.banner_message = Some((lang_msg, false, 2.0));
            }

            // Open selected file/folder in default host application
            if is_key_pressed(KeyCode::O) {
                gui.app.open_current_selected();
                if let Some((msg, is_err)) = gui.app.status_message.take() {
                    gui.banner_message = Some((msg, is_err, 2.5));
                }
            }

            // Spotlight search trigger
            if is_key_pressed(KeyCode::Slash) {
                while get_char_pressed().is_some() {}
                gui.app.start_search();
            }

            // Scrapyard toggle
            if is_key_pressed(KeyCode::J) {
                gui.app.toggle_scrapyard();
                gui.avatar.x = GuiApp::STREET_MARGIN_LEFT;
                gui.avatar.vx = 0.0;
                gui.avatar.vy = 0.0;
                gui.camera_x = 0.0;
                gui.particles.clear();
                if let Some((msg, is_err)) = gui.app.status_message.take() {
                    gui.banner_message = Some((msg, is_err, 3.0));
                }
            }

            // Bulldozer batch wipe in Scrapyard mode
            if is_key_pressed(KeyCode::C) && gui.app.is_in_scrapyard() {
                for p in &placements {
                    let ground = GuiApp::ground_y(screen_h);
                    let top_y = ground - p.height;
                    gui.spawn_demolition_particles(
                        p.x,
                        top_y,
                        p.width,
                        p.height,
                        Color::new(0.85, 0.45, 0.18, 1.0),
                    );
                }
                let _ = gui.app.execute_bulldozer();
                gui.avatar.x = GuiApp::STREET_MARGIN_LEFT;
                gui.avatar.vx = 0.0;
                gui.camera_x = 0.0;
                if let Some((msg, is_err)) = gui.app.status_message.take() {
                    gui.banner_message = Some((msg, is_err, 3.5));
                }
            }
            // Demolition trigger
            let trigger_demolish = is_key_pressed(KeyCode::Delete)
                || ((is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Delete))
                    && (is_key_down(KeyCode::LeftControl)
                        || is_key_down(KeyCode::LeftShift)
                        || is_key_pressed(KeyCode::Delete)));

            if trigger_demolish {
                if let Some(child) = gui.app.current_selected_child() {
                    if child.is_system_protected {
                        let warn = gui.app.language.protected_rejection_notice().to_string();
                        gui.banner_message = Some((warn, true, 3.5));
                    } else {
                        gui.modal = Some(GuiDemolishModal {
                            target_index: gui.app.nav.selected_index,
                            target_name: child.name.clone(),
                            target_size_formatted: format_bytes(child.size),
                            is_protected: child.is_system_protected,
                        });
                    }
                }
            }

            // Fullscreen toggle
            if is_key_pressed(KeyCode::F11) {
                gui.is_fullscreen = !gui.is_fullscreen;
                set_fullscreen(gui.is_fullscreen);
            }

            // Quit
            if is_key_pressed(KeyCode::Q) {
                break;
            }
        }

        // 3. Update Physics & Camera
        gui.update_physics(dt, screen_w, screen_h, max_street_x);
        gui.update_selected_building_from_avatar(&placements);

        // 4. Render Scene
        render_night_sky(
            &gui.stars,
            screen_w,
            screen_h,
            gui.camera_x,
            time,
            gui.app.is_in_scrapyard(),
        );
        render_street(screen_w, screen_h, gui.camera_x);
        render_street_radar(&placements, gui.avatar.x, max_street_x, screen_w, time);

        // Render Buildings
        let curr_node = gui.app.current_node();
        let font_ref = gui.font.as_ref();
        for p in &placements {
            let is_selected = p.node_index == gui.app.nav.selected_index;
            if let Some(child) = curr_node.children.get(p.node_index) {
                render_building(
                    p,
                    child,
                    is_selected,
                    screen_h,
                    gui.camera_x,
                    time,
                    font_ref,
                );
            }
        }

        // Render Avatar & Particles
        render_avatar(&gui.avatar, screen_h, gui.camera_x);
        render_particles(&gui.particles, gui.camera_x);

        // Render RPG HUD
        let selected_child = gui.app.current_selected_child();
        render_rpg_hud(&gui.app, selected_child, screen_w, screen_h, font_ref);

        // Render Search Overlay if active
        if let Some(ref q) = gui.app.search_query {
            let matched_node = gui.app.current_selected_child();
            render_search_overlay(q, matched_node, gui.app.language, screen_w, font_ref);
        }
        // Render Modal or Banner if active
        if let Some(m) = &gui.modal {
            render_demolish_modal(m, gui.app.language, screen_w, screen_h, font_ref);
        } else if let Some((msg, is_err, _)) = &gui.banner_message {
            render_banner(msg, *is_err, screen_w, font_ref);
        }

        next_frame().await;
    }
}
