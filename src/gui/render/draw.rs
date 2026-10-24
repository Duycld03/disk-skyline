use crate::model::BuildingCategory;
use macroquad::prelude::*;
use macroquad::text::Font;

pub fn category_color(cat: BuildingCategory) -> Color {
    match cat {
        BuildingCategory::Directory => Color::new(0.25, 0.55, 0.95, 1.0), // Vibrant Blue
        BuildingCategory::Code => Color::new(0.20, 0.85, 0.40, 1.0),      // Neon Green
        BuildingCategory::Media => Color::new(0.92, 0.25, 0.75, 1.0),     // Cyberpunk Magenta
        BuildingCategory::Archive => Color::new(0.95, 0.75, 0.15, 1.0),   // Amber Industrial
        BuildingCategory::Binary => Color::new(0.95, 0.35, 0.20, 1.0),    // Molten Orange
        BuildingCategory::Document => Color::new(0.85, 0.88, 0.92, 1.0),  // Silver White
        BuildingCategory::Restricted => Color::new(0.90, 0.15, 0.15, 1.0), // Danger Red
        BuildingCategory::Junk => Color::new(0.85, 0.45, 0.18, 1.0),      // Rust Amber / Scrapyard
        BuildingCategory::Other => Color::new(0.60, 0.65, 0.72, 1.0),     // Industrial Slate
    }
}

pub fn draw_text_f(text: &str, x: f32, y: f32, font_size: f32, color: Color, font: Option<&Font>) {
    if let Some(f) = font {
        macroquad::text::draw_text_ex(
            text,
            x,
            y,
            macroquad::text::TextParams {
                font: Some(f),
                font_size: font_size as u16,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                color,
                rotation: 0.0,
            },
        );
    } else {
        macroquad::text::draw_text(text, x, y, font_size, color);
    }
}

pub fn measure_text_f(text: &str, font_size: f32, font: Option<&Font>) -> f32 {
    macroquad::text::measure_text(text, font, font_size as u16, 1.0).width
}

pub fn fit_text_to_width(text: &str, max_w: f32, font_size: f32, font: Option<&Font>) -> String {
    if max_w <= 8.0 {
        return String::new();
    }
    let full_w = measure_text_f(text, font_size, font);
    if full_w <= max_w {
        return text.to_string();
    }

    let mut truncated = text.to_string();
    while !truncated.is_empty() {
        truncated.pop();
        let candidate = format!("{}..", truncated);
        if measure_text_f(&candidate, font_size, font) <= max_w {
            return candidate;
        }
    }
    String::new()
}

pub fn building_category_badge(
    cat: BuildingCategory,
    is_dir: bool,
    is_protected: bool,
) -> &'static str {
    if is_protected {
        "[!]"
    } else if is_dir {
        "[DIR]"
    } else {
        match cat {
            BuildingCategory::Code => "[CODE]",
            BuildingCategory::Media => "[MEDIA]",
            BuildingCategory::Archive => "[ZIP]",
            BuildingCategory::Binary => "[BIN]",
            BuildingCategory::Document => "[DOC]",
            BuildingCategory::Restricted => "[LOCK]",
            BuildingCategory::Junk => "[SCRAP]",
            BuildingCategory::Directory => "[DIR]",
            BuildingCategory::Other => "[FILE]",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_category_badges() {
        assert_eq!(
            building_category_badge(BuildingCategory::Directory, true, false),
            "[DIR]"
        );
        assert_eq!(
            building_category_badge(BuildingCategory::Code, false, false),
            "[CODE]"
        );
        assert_eq!(
            building_category_badge(BuildingCategory::Media, false, false),
            "[MEDIA]"
        );
        assert_eq!(
            building_category_badge(BuildingCategory::Archive, false, false),
            "[ZIP]"
        );
        assert_eq!(
            building_category_badge(BuildingCategory::Binary, false, false),
            "[BIN]"
        );
        assert_eq!(
            building_category_badge(BuildingCategory::Document, false, false),
            "[DOC]"
        );
        assert_eq!(
            building_category_badge(BuildingCategory::Code, false, true),
            "[!]"
        );
        assert_eq!(
            building_category_badge(BuildingCategory::Junk, false, false),
            "[SCRAP]"
        );
    }

    #[test]
    fn test_fit_text_to_width_boundary() {
        assert_eq!(
            fit_text_to_width("very_long_building_name.rs", 5.0, 11.0, None),
            ""
        );
    }
}
