use crate::app::App;
use crate::skyline::calculate_building_height;
use macroquad::prelude::*;
use macroquad::text::Font;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Avatar {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub is_grounded: bool,
    pub facing: Direction,
    pub walk_timer: f32,
}

impl Default for Avatar {
    fn default() -> Self {
        Self {
            x: 100.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            is_grounded: true,
            facing: Direction::Right,
            walk_timer: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color: Color,
    pub size: f32,
    pub life: f32,
    pub max_life: f32,
}

#[derive(Debug, Clone)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub base_alpha: f32,
    pub twinkle_speed: f32,
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct BuildingPlacement2D {
    pub node_index: usize,
    pub x: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiDemolishModal {
    pub target_index: usize,
    pub target_name: String,
    pub target_size_formatted: String,
    pub is_protected: bool,
}

pub struct GuiApp {
    pub app: App,
    pub avatar: Avatar,
    pub camera_x: f32,
    pub particles: Vec<Particle>,
    pub stars: Vec<Star>,
    pub modal: Option<GuiDemolishModal>,
    pub banner_message: Option<(String, bool, f32)>, // (msg, is_error, timer)
    pub is_fullscreen: bool,
    pub font: Option<Font>,
    pub backspace_timer: f32,
}

impl GuiApp {
    pub const STREET_MARGIN_LEFT: f32 = 60.0;
    pub const BUILDING_GAP: f32 = 30.0;
    pub const MIN_BUILDING_WIDTH: f32 = 90.0;
    pub const MAX_BUILDING_WIDTH: f32 = 140.0;
    pub const MIN_BUILDING_HEIGHT_PX: u16 = 50;
    pub const MAX_BUILDING_HEIGHT_PX: u16 = 380;
    pub const AVATAR_WIDTH: f32 = 24.0;
    pub const AVATAR_HEIGHT: f32 = 38.0;
    pub const WALK_SPEED: f32 = 240.0;
    pub const JUMP_VELOCITY: f32 = -480.0;
    pub const GRAVITY: f32 = 1100.0;

    pub fn new(app: App) -> Self {
        // Pre-generate stars for night sky
        let mut stars = Vec::with_capacity(120);
        let mut seed: u64 = 0xDEADBEEF;
        for _ in 0..120 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let rx = ((seed >> 16) % 3840) as f32;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let ry = ((seed >> 16) % 400) as f32;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let alpha = 0.3 + (((seed >> 16) % 70) as f32) / 100.0;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let speed = 1.0 + (((seed >> 16) % 40) as f32) / 10.0;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let size = if (seed >> 16).is_multiple_of(10) {
                2.5
            } else {
                1.5
            };
            stars.push(Star {
                x: rx,
                y: ry,
                base_alpha: alpha,
                twinkle_speed: speed,
                size,
            });
        }

        Self {
            app,
            avatar: Avatar {
                y: Self::ground_y(720.0) - Self::AVATAR_HEIGHT,
                ..Avatar::default()
            },
            camera_x: 0.0,
            particles: Vec::new(),
            stars,
            modal: None,
            banner_message: None,
            is_fullscreen: false,
            font: None,
            backspace_timer: 0.0,
        }
    }

    pub fn ground_y(screen_h: f32) -> f32 {
        screen_h - 170.0
    }

    /// Computes 2D layout placements for all children in the current directory
    pub fn compute_placements(&self, screen_h: f32) -> Vec<BuildingPlacement2D> {
        let current_node = self.app.current_node();
        let children = &current_node.children;
        if children.is_empty() {
            return Vec::new();
        }

        let max_size = children.iter().map(|c| c.size).max().unwrap_or(1);
        let max_h = ((screen_h - 260.0).max(120.0) as u16).min(Self::MAX_BUILDING_HEIGHT_PX);

        let mut placements = Vec::with_capacity(children.len());
        let mut curr_x = Self::STREET_MARGIN_LEFT;

        for (idx, child) in children.iter().enumerate() {
            let height_px = calculate_building_height(
                child.size,
                max_size,
                Self::MIN_BUILDING_HEIGHT_PX,
                max_h,
            ) as f32;

            let name_len = child.name.chars().count() as f32;
            let width_px =
                (name_len * 9.0 + 35.0).clamp(Self::MIN_BUILDING_WIDTH, Self::MAX_BUILDING_WIDTH);

            placements.push(BuildingPlacement2D {
                node_index: idx,
                x: curr_x,
                width: width_px,
                height: height_px,
            });

            curr_x += width_px + Self::BUILDING_GAP;
        }

        placements
    }

    /// Finds the building index that the avatar is currently standing in front of
    pub fn update_selected_building_from_avatar(&mut self, placements: &[BuildingPlacement2D]) {
        if placements.is_empty() {
            return;
        }

        let avatar_mid = self.avatar.x + Self::AVATAR_WIDTH / 2.0;

        // First check if avatar is directly in front of a building
        for p in placements {
            if avatar_mid >= p.x && avatar_mid <= p.x + p.width {
                self.app.nav.selected_index = p.node_index;
                return;
            }
        }

        // If in a gap or before/after street, pick the closest building center
        let mut closest_idx = placements[0].node_index;
        let mut min_dist = f32::MAX;

        for p in placements {
            let building_mid = p.x + p.width / 2.0;
            let dist = (avatar_mid - building_mid).abs();
            if dist < min_dist {
                min_dist = dist;
                closest_idx = p.node_index;
            }
        }

        self.app.nav.selected_index = closest_idx;
    }

    /// Advances avatar physics, jumping, walking velocity, and gravity
    pub fn update_physics(&mut self, dt: f32, screen_w: f32, screen_h: f32, max_street_x: f32) {
        let ground = Self::ground_y(screen_h);
        let floor_y = ground - Self::AVATAR_HEIGHT;

        // Apply gravity
        if !self.avatar.is_grounded {
            self.avatar.vy += Self::GRAVITY * dt;
        }

        // Update positions
        self.avatar.x += self.avatar.vx * dt;
        self.avatar.y += self.avatar.vy * dt;

        // Ground collision
        if self.avatar.y >= floor_y {
            self.avatar.y = floor_y;
            self.avatar.vy = 0.0;
            self.avatar.is_grounded = true;
        } else {
            self.avatar.is_grounded = false;
        }

        // Street boundary clamping
        let street_limit = max_street_x.max(screen_w);
        self.avatar.x = self
            .avatar
            .x
            .clamp(10.0, street_limit - Self::AVATAR_WIDTH - 10.0);

        // Walk animation timer
        if self.avatar.vx.abs() > 10.0 && self.avatar.is_grounded {
            self.avatar.walk_timer += dt * 10.0;
        } else if self.avatar.is_grounded {
            self.avatar.walk_timer = 0.0;
        }

        // Smooth camera lerp tracking avatar
        let target_cam_x = self.avatar.x + Self::AVATAR_WIDTH / 2.0 - screen_w / 2.0;
        let cam_lerp = (dt * 5.0).clamp(0.0, 1.0);
        self.camera_x += (target_cam_x - self.camera_x) * cam_lerp;
        self.camera_x = self.camera_x.max(0.0);

        // Update banner message timer
        let mut clear_banner = false;
        if let Some((_, _, timer)) = &mut self.banner_message {
            *timer -= dt;
            if *timer <= 0.0 {
                clear_banner = true;
            }
        }
        if clear_banner {
            self.banner_message = None;
        }
        // Update particles
        let mut i = 0;
        while i < self.particles.len() {
            let p = &mut self.particles[i];
            p.vy += 700.0 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life += dt;

            // Bounce off ground
            if p.y >= ground {
                p.y = ground;
                p.vy = -p.vy * 0.4;
                p.vx *= 0.7;
            }

            if p.life >= p.max_life {
                self.particles.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Spawns demolition explosion debris particles
    pub fn spawn_demolition_particles(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let mut seed: u64 = (x as u64) ^ ((y as u64) << 16) ^ (self.particles.len() as u64);
        for _ in 0..70 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let px = x + (((seed >> 16) % 1000) as f32 / 1000.0) * w;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let py = y + (((seed >> 16) % 1000) as f32 / 1000.0) * h;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let vx = -220.0 + (((seed >> 16) % 440) as f32);
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let vy = -380.0 + (((seed >> 16) % 300) as f32);
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let size = 3.0 + (((seed >> 16) % 6) as f32);
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let max_life = 1.0 + (((seed >> 16) % 80) as f32) / 100.0;

            // Mix building color with smoke gray/white
            let p_color = if (seed >> 16).is_multiple_of(3) {
                Color::new(0.9, 0.9, 0.9, 0.9)
            } else if (seed >> 16) % 3 == 1 {
                Color::new(0.95, 0.5, 0.1, 1.0)
            } else {
                color
            };

            self.particles.push(Particle {
                x: px,
                y: py,
                vx,
                vy,
                color: p_color,
                size,
                life: 0.0,
                max_life,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BuildingCategory, BuildingNode};
    use crate::scanner::ScannerOptions;
    use std::path::PathBuf;

    fn create_mock_gui_app() -> GuiApp {
        let child1 = BuildingNode {
            name: "tower1.rs".into(),
            path: PathBuf::from("/test/tower1.rs"),
            is_dir: false,
            is_hidden: false,
            is_system_protected: false,
            size: 10_000,
            file_type: BuildingCategory::Code,
            children: vec![],
            modified_secs_ago: None,
            purge_action: None,
        };
        let child2 = BuildingNode {
            name: "assets".into(),
            path: PathBuf::from("/test/assets"),
            is_dir: true,
            is_hidden: false,
            is_system_protected: false,
            size: 50_000,
            file_type: BuildingCategory::Directory,
            children: vec![],
            modified_secs_ago: None,
            purge_action: None,
        };
        let root = BuildingNode {
            name: "root".into(),
            path: PathBuf::from("/test"),
            is_dir: true,
            is_hidden: false,
            is_system_protected: false,
            size: 60_000,
            file_type: BuildingCategory::Directory,
            children: vec![child1, child2],
            modified_secs_ago: None,
            purge_action: None,
        };

        let app = App::new(
            root,
            ScannerOptions {
                max_depth: 3,
                sort_mode: crate::model::SortMode::default(),
            },
        );
        GuiApp::new(app)
    }

    #[test]
    fn test_avatar_default_state_and_physics() {
        let mut gui = create_mock_gui_app();
        assert!(gui.avatar.is_grounded);
        assert_eq!(gui.avatar.facing, Direction::Right);

        // Moving right
        gui.avatar.vx = GuiApp::WALK_SPEED;
        gui.update_physics(0.1, 1280.0, 720.0, 2000.0);
        assert!(gui.avatar.x > 100.0);
        assert!(gui.avatar.is_grounded);
        assert!(gui.avatar.walk_timer > 0.0);
    }

    #[test]
    fn test_avatar_jump_and_gravity() {
        let mut gui = create_mock_gui_app();
        let ground = GuiApp::ground_y(720.0);
        let floor_y = ground - GuiApp::AVATAR_HEIGHT;
        gui.avatar.y = floor_y;

        // Execute jump
        gui.avatar.vy = GuiApp::JUMP_VELOCITY;
        gui.avatar.is_grounded = false;

        gui.update_physics(0.05, 1280.0, 720.0, 2000.0);
        assert!(gui.avatar.y < floor_y);
        assert!(!gui.avatar.is_grounded);

        // Simulate falling back down
        for _ in 0..30 {
            gui.update_physics(0.05, 1280.0, 720.0, 2000.0);
        }
        assert_eq!(gui.avatar.y, floor_y);
        assert!(gui.avatar.is_grounded);
    }

    #[test]
    fn test_compute_placements_and_selected_building() {
        let mut gui = create_mock_gui_app();
        let placements = gui.compute_placements(720.0);
        assert_eq!(placements.len(), 2);
        assert_eq!(placements[0].node_index, 0);
        assert_eq!(placements[1].node_index, 1);
        assert!(placements[1].x > placements[0].x);

        // Position avatar in front of building 0
        gui.avatar.x = placements[0].x + 10.0;
        gui.update_selected_building_from_avatar(&placements);
        assert_eq!(gui.app.nav.selected_index, 0);

        // Move avatar in front of building 1
        gui.avatar.x = placements[1].x + 10.0;
        gui.update_selected_building_from_avatar(&placements);
        assert_eq!(gui.app.nav.selected_index, 1);
    }

    #[test]
    fn test_demolition_particles_spawning() {
        let mut gui = create_mock_gui_app();
        assert_eq!(gui.particles.len(), 0);
        gui.spawn_demolition_particles(100.0, 200.0, 80.0, 150.0, Color::new(0.2, 0.8, 0.4, 1.0));
        assert_eq!(gui.particles.len(), 70);

        // Update physics to verify particle aging
        gui.update_physics(0.1, 1280.0, 720.0, 2000.0);
        assert_eq!(gui.particles.len(), 70);
        assert!(gui.particles[0].life > 0.0);
    }

    #[test]
    fn test_d_key_walking_and_x_demolition_contract() {
        let mut gui = create_mock_gui_app();
        assert_eq!(gui.avatar.facing, Direction::Right);
        assert_eq!(gui.modal, None);

        // D key sets positive walking velocity without triggering demolition modal
        gui.avatar.vx = GuiApp::WALK_SPEED;
        gui.avatar.facing = Direction::Right;
        gui.update_physics(0.1, 1280.0, 720.0, 2000.0);
        assert!(
            gui.avatar.x > 100.0,
            "D walking must advance avatar x position"
        );
        assert_eq!(
            gui.modal, None,
            "Walking right must never trigger demolition modal"
        );
    }
}
