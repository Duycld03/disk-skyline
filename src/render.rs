#![allow(clippy::too_many_arguments)]
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crate::model::{BuildingCategory, BuildingNode};
use crate::skyline::SkylinePlacement;

pub struct BuildingTheme {
    pub border_color: Color,
    pub window_color: Color,
    pub roof_pattern: &'static str,
    pub floor_pattern: &'static str,
    pub door_pattern: &'static str,
}

impl BuildingTheme {
    pub fn for_category(
        category: BuildingCategory,
        is_selected: bool,
        language: crate::i18n::Language,
    ) -> Self {
        if is_selected {
            return BuildingTheme {
                border_color: Color::Yellow,
                window_color: Color::White,
                roof_pattern: "^-*-^",
                floor_pattern: "[*]",
                door_pattern: "[ENTER]",
            };
        }

        match category {
            BuildingCategory::Directory => BuildingTheme {
                border_color: Color::Cyan,
                window_color: Color::LightCyan,
                roof_pattern: "/---\\",
                floor_pattern: "[:]",
                door_pattern: "[ ]",
            },
            BuildingCategory::Code => BuildingTheme {
                border_color: Color::Green,
                window_color: Color::LightGreen,
                roof_pattern: "<--->",
                floor_pattern: "<>",
                door_pattern: "[#]",
            },
            BuildingCategory::Media => BuildingTheme {
                border_color: Color::Magenta,
                window_color: Color::LightMagenta,
                roof_pattern: "[* *]",
                floor_pattern: "[o]",
                door_pattern: "[~]",
            },
            BuildingCategory::Archive => BuildingTheme {
                border_color: Color::Rgb(200, 140, 40), // Amber
                window_color: Color::Yellow,
                roof_pattern: "[===]",
                floor_pattern: "###",
                door_pattern: "[=]",
            },
            BuildingCategory::Binary => BuildingTheme {
                border_color: Color::Red,
                window_color: Color::LightRed,
                roof_pattern: "+T T+",
                floor_pattern: "|:|",
                door_pattern: "[!]",
            },
            BuildingCategory::Document => BuildingTheme {
                border_color: Color::Blue,
                window_color: Color::LightBlue,
                roof_pattern: "+---+",
                floor_pattern: "| |",
                door_pattern: "[=]",
            },
            BuildingCategory::Restricted => BuildingTheme {
                border_color: Color::Rgb(180, 50, 50), // Hazard Red/Dark
                window_color: Color::Yellow,
                roof_pattern: "/!/!\\",
                floor_pattern: "///",
                door_pattern: match language {
                    crate::i18n::Language::Vi => "[CẤM]",
                    crate::i18n::Language::En => "[BAN]",
                },
            },
            BuildingCategory::Other => BuildingTheme {
                border_color: Color::Gray,
                window_color: Color::White,
                roof_pattern: "+---+",
                floor_pattern: "| |",
                door_pattern: "[ ]",
            },
        }
    }
}

/// Renders the entire street skyline onto a Ratatui buffer
pub fn render_skyline(
    nodes: &[BuildingNode],
    placements: &[SkylinePlacement],
    selected_index: usize,
    viewport_offset_x: usize,
    area: Rect,
    buf: &mut Buffer,
    language: crate::i18n::Language,
) {
    if area.width < 10 || area.height < 6 {
        return;
    }

    // Determine street baseline row (leaving room for street line, names, and sizes below)
    let ground_row = area.y + area.height.saturating_sub(4);

    // 1. Draw horizontal street line
    let street_style = Style::default().fg(Color::DarkGray);
    for x in area.left()..area.right() {
        buf[(x, ground_row)].set_symbol("=").set_style(street_style);
    }

    // 2. Render each building in placement order
    for (idx, placement) in placements.iter().enumerate() {
        if placement.node_index >= nodes.len() {
            continue;
        }
        let node = &nodes[placement.node_index];
        let is_selected = idx == selected_index;

        let screen_start_col =
            (area.x as usize + placement.start_col) as isize - viewport_offset_x as isize;
        let screen_end_col = screen_start_col + placement.width as isize;

        // Skip buildings completely outside the visible viewport
        if screen_end_col <= area.left() as isize || screen_start_col >= area.right() as isize {
            continue;
        }

        render_single_building(
            node,
            placement,
            screen_start_col,
            ground_row,
            is_selected,
            area,
            buf,
            language,
        );
    }
}

fn render_single_building(
    node: &BuildingNode,
    placement: &SkylinePlacement,
    screen_start_col: isize,
    ground_row: u16,
    is_selected: bool,
    area: Rect,
    buf: &mut Buffer,
    language: crate::i18n::Language,
) {
    let theme = BuildingTheme::for_category(node.file_type, is_selected, language);
    let wall_style = if is_selected {
        Style::default()
            .fg(theme.border_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.border_color)
    };
    let window_style = Style::default().fg(theme.window_color);

    let b_width = placement.width as usize;
    let top_row = ground_row.saturating_sub(placement.height);

    // Draw Roof
    if top_row >= area.top() && top_row < ground_row {
        draw_roof(
            screen_start_col,
            top_row,
            b_width,
            node.file_type,
            node.is_system_protected,
            &theme,
            wall_style,
            area,
            buf,
        );
    }

    // Draw Floors / Body
    for row in (top_row + 1)..ground_row {
        if row >= area.top() && row < area.bottom() {
            draw_floor(
                screen_start_col,
                row,
                b_width,
                &theme,
                wall_style,
                window_style,
                area,
                buf,
            );
        }
    }

    // Draw Ground / Entrance Foundation
    let base_row = ground_row.saturating_sub(1);
    if base_row >= area.top() && base_row > top_row {
        draw_entrance(
            screen_start_col,
            base_row,
            b_width,
            node.is_system_protected,
            &theme,
            wall_style,
            area,
            buf,
        );
    }
    // Draw Name & Size below the street
    let name_row = ground_row + 1;
    let size_row = ground_row + 2;
    let marker_row = ground_row + 3;

    if name_row < area.bottom() {
        let label = truncate_string(&node.name, b_width);
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        draw_text_centered(
            screen_start_col,
            name_row,
            b_width,
            &label,
            style,
            area,
            buf,
        );
    }

    if size_row < area.bottom() {
        let size_str = node.formatted_size();
        let size_style = Style::default().fg(Color::DarkGray);
        draw_text_centered(
            screen_start_col,
            size_row,
            b_width,
            &size_str,
            size_style,
            area,
            buf,
        );
    }

    // Selection cursor arrow
    if is_selected && marker_row < area.bottom() {
        draw_text_centered(
            screen_start_col,
            marker_row,
            b_width,
            language.selected_badge(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            area,
            buf,
        );
    }
}
fn draw_roof(
    start_x: isize,
    y: u16,
    width: usize,
    category: BuildingCategory,
    is_protected: bool,
    theme: &BuildingTheme,
    style: Style,
    area: Rect,
    buf: &mut Buffer,
) {
    let pattern = if is_protected {
        "[LOCK]"
    } else if category == BuildingCategory::Restricted {
        "/!/!\\"
    } else {
        theme.roof_pattern
    };
    let full_roof = if width <= 2 {
        "+".to_string()
    } else {
        let inner_len = width.saturating_sub(2);
        let mut inner = String::new();
        while inner.len() < inner_len {
            inner.push_str(pattern);
        }
        inner.truncate(inner_len);
        format!("+{}+", inner)
    };

    draw_row_slice(start_x, y, &full_roof, style, area, buf);
}

fn draw_floor(
    start_x: isize,
    y: u16,
    width: usize,
    theme: &BuildingTheme,
    wall_style: Style,
    window_style: Style,
    area: Rect,
    buf: &mut Buffer,
) {
    if width <= 2 {
        draw_row_slice(start_x, y, "||", wall_style, area, buf);
        return;
    }

    // Left wall
    draw_char(start_x, y, '|', wall_style, area, buf);

    // Windows / Interior
    let inner_width = width - 2;
    let mut interior = String::new();
    while interior.len() < inner_width {
        interior.push_str(theme.floor_pattern);
        interior.push(' ');
    }
    interior.truncate(inner_width);
    draw_row_slice(start_x + 1, y, &interior, window_style, area, buf);

    // Right wall
    draw_char(start_x + width as isize - 1, y, '|', wall_style, area, buf);
}

fn draw_entrance(
    start_x: isize,
    y: u16,
    width: usize,
    is_protected: bool,
    theme: &BuildingTheme,
    style: Style,
    area: Rect,
    buf: &mut Buffer,
) {
    if width <= 4 {
        draw_row_slice(start_x, y, "|[]|", style, area, buf);
        return;
    }

    let door = if is_protected {
        "[SYS]"
    } else {
        theme.door_pattern
    };
    let inner_width = width.saturating_sub(2);
    let padding = inner_width.saturating_sub(door.len()) / 2;
    let mut row = String::with_capacity(width);
    row.push('|');
    for _ in 0..padding {
        row.push(' ');
    }
    row.push_str(door);
    while row.len() < width - 1 {
        row.push(' ');
    }
    row.push('|');

    draw_row_slice(start_x, y, &row, style, area, buf);
}

fn draw_row_slice(start_x: isize, y: u16, text: &str, style: Style, area: Rect, buf: &mut Buffer) {
    for (i, ch) in text.chars().enumerate() {
        let x = start_x + i as isize;
        draw_char(x, y, ch, style, area, buf);
    }
}

fn draw_char(x: isize, y: u16, ch: char, style: Style, area: Rect, buf: &mut Buffer) {
    if x >= area.left() as isize
        && x < area.right() as isize
        && y >= area.top()
        && y < area.bottom()
    {
        buf[(x as u16, y)].set_char(ch).set_style(style);
    }
}

fn draw_text_centered(
    start_x: isize,
    y: u16,
    width: usize,
    text: &str,
    style: Style,
    area: Rect,
    buf: &mut Buffer,
) {
    let text_len = text.chars().count();
    let offset = width.saturating_sub(text_len) / 2;
    draw_row_slice(start_x + offset as isize, y, text, style, area, buf);
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        let mut res: String = s.chars().take(max_len - 2).collect();
        res.push_str("..");
        res
    } else {
        s.chars().take(max_len).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_render_skyline_to_buffer() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 20));
        let nodes = vec![
            BuildingNode::new(
                "src".into(),
                PathBuf::from("src"),
                true,
                false,
                50_000,
                vec![],
            ),
            BuildingNode::new(
                ".git".into(),
                PathBuf::from(".git"),
                true,
                true,
                20_000,
                vec![],
            ),
        ];

        let placements = vec![
            SkylinePlacement {
                node_index: 0,
                start_col: 0,
                width: 10,
                height: 8,
            },
            SkylinePlacement {
                node_index: 1,
                start_col: 11,
                width: 10,
                height: 5,
            },
        ];

        render_skyline(
            &nodes,
            &placements,
            0,
            0,
            Rect::new(0, 0, 80, 20),
            &mut buf,
            crate::i18n::Language::Vi,
        );
        assert_eq!(buf[(0, 16)].symbol(), "=");
        assert_eq!(buf[(5, 16)].symbol(), "=");

        // First building (.src) is selected (selected_index: 0), so it should have Yellow style
        let roof_cell = &buf[(0, 16 - 8)];
        assert_eq!(roof_cell.symbol(), "+");
        assert_eq!(roof_cell.style().fg, Some(Color::Yellow));

        // Second building (.git) is restricted/bunker, check that it renders
        let bunker_cell = &buf[(11, 16 - 5)];
        assert_eq!(bunker_cell.symbol(), "+");
    }
}
