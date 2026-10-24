use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::render::render_skyline;
use crate::skyline::layout_skyline;

pub fn draw_ui(app: &mut App, frame: &mut Frame) {
    let size = frame.area();

    // Divide main screen into: Header (3), Skyline (min 6), Minimap (1), HUD (4)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header & Breadcrumbs
            Constraint::Min(6),    // Skyline Viewport
            Constraint::Length(1), // Minimap
            Constraint::Length(4), // HUD & Keybindings
        ])
        .split(size);

    let header_area = chunks[0];
    let skyline_area = chunks[1];
    let minimap_area = chunks[2];
    let hud_area = chunks[3];

    // Compute skyline placements for the current directory
    let placements = {
        let current_node = app.current_node();
        layout_skyline(&current_node.children, skyline_area.height)
    };

    // Update camera viewport tracking the selected building
    app.update_camera_viewport(skyline_area.width, &placements);

    // 1. Render Header
    render_header(app, header_area, frame);

    // 2. Render Skyline
    let current_node = app.current_node();
    let buf = frame.buffer_mut();
    render_skyline(
        &current_node.children,
        &placements,
        app.nav.selected_index,
        app.nav.viewport_offset_x,
        skyline_area,
        buf,
        app.language,
    );
    // 3. Render Minimap
    render_minimap(app, &placements, skyline_area.width, minimap_area, frame);

    // 4. Render HUD Inspection Card
    render_hud(app, hud_area, frame);

    // 5. Render Demolition Modal if active
    if let Some(state) = &app.demolish_state {
        crate::demolish::render_demolish_modal(state, app.language, frame);
    }
}

fn render_header(app: &App, area: Rect, frame: &mut Frame) {
    let trail = app.breadcrumbs().join(" / ");
    let current_node = app.current_node();
    let lang = app.language;

    let avenue_label = match lang {
        crate::i18n::Language::Vi => "Đại lộ: ",
        crate::i18n::Language::En => "Avenue: ",
    };

    let title_line = Line::from(vec![
        Span::styled(
            " DISK SKYLINE ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(avenue_label, Style::default().fg(Color::DarkGray)),
        Span::styled(
            trail,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let status_info = if let Some((msg, is_err)) = &app.status_message {
        let color = if *is_err { Color::Red } else { Color::Green };
        Span::styled(
            format!(" [{}]", msg),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )
    } else {
        let buildings_word = match lang {
            crate::i18n::Language::Vi => "công trình",
            crate::i18n::Language::En => "buildings",
        };
        let total_word = match lang {
            crate::i18n::Language::Vi => "Tổng: ",
            crate::i18n::Language::En => "Total: ",
        };
        Span::styled(
            format!(
                " [{}]  {} {} ({}{})",
                app.nav.sort_mode.label_lang(lang),
                current_node.children.len(),
                buildings_word,
                total_word,
                current_node.formatted_size()
            ),
            Style::default().fg(Color::DarkGray),
        )
    };

    let zoning_word = match lang {
        crate::i18n::Language::Vi => "Quy hoạch: ",
        crate::i18n::Language::En => "Zoning: ",
    };

    let sub_line = Line::from(vec![
        Span::styled(zoning_word, Style::default().fg(Color::DarkGray)),
        status_info,
    ]);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(vec![title_line, sub_line]).block(block);
    frame.render_widget(paragraph, area);
}

fn render_minimap(
    app: &App,
    placements: &[crate::skyline::SkylinePlacement],
    screen_width: u16,
    area: Rect,
    frame: &mut Frame,
) {
    if area.width < 10 || placements.is_empty() {
        return;
    }

    let total_street_width = placements
        .last()
        .map(|p| p.start_col + p.width as usize)
        .unwrap_or(1);

    let map_cols = area.width as usize;
    let mut minimap_chars = vec!['='; map_cols];

    // Compute active camera viewport window on minimap
    let start_pct = (app.nav.viewport_offset_x as f64 / total_street_width as f64).clamp(0.0, 1.0);
    let end_pct = ((app.nav.viewport_offset_x as f64 + screen_width as f64)
        / total_street_width as f64)
        .clamp(0.0, 1.0);

    let map_start = (start_pct * map_cols as f64).round() as usize;
    let map_end = (end_pct * map_cols as f64).round() as usize;

    for ch in minimap_chars[map_start..map_end.min(map_cols)].iter_mut() {
        *ch = '#';
    }

    // Mark current selected building on minimap
    if let Some(p) = placements.get(app.nav.selected_index) {
        let cur_pct = (p.start_col as f64 / total_street_width as f64).clamp(0.0, 1.0);
        let cur_col = (cur_pct * (map_cols - 1) as f64).round() as usize;
        if cur_col < map_cols {
            minimap_chars[cur_col] = '*';
        }
    }

    let map_string: String = minimap_chars.into_iter().collect();
    let paragraph = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{}[", app.language.street_label()),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(map_string, Style::default().fg(Color::Cyan)),
        Span::styled("]", Style::default().fg(Color::DarkGray)),
    ]));

    frame.render_widget(paragraph, area);
}

fn render_hud(app: &App, area: Rect, frame: &mut Frame) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let lang = app.language;
    let selected_opt = app.current_selected_child();

    let line1 = if let Some(child) = selected_opt {
        let pct = if app.current_node().size > 0 {
            (child.size as f64 / app.current_node().size as f64) * 100.0
        } else {
            0.0
        };
        let restricted_tag = if child.is_system_protected {
            Span::styled(
                format!(" {} ", lang.protected_tag()),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            )
        } else if child.is_hidden {
            Span::styled(
                format!(" {} ", lang.bunker_tag()),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        };

        let avenue_pct = match lang {
            crate::i18n::Language::Vi => format!("({:.1}% phố)", pct),
            crate::i18n::Language::En => format!("({:.1}% avenue)", pct),
        };

        Line::from(vec![
            Span::styled(lang.building_label(), Style::default().fg(Color::DarkGray)),
            Span::styled(
                &child.name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(
                child.file_type.display_label_lang(lang),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" | "),
            Span::styled(lang.size_label(), Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} {}", child.formatted_size(), avenue_pct),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            restricted_tag,
        ])
    } else {
        let empty_msg = match lang {
            crate::i18n::Language::Vi => "Không có công trình nào trong phố này.",
            crate::i18n::Language::En => "No buildings in this avenue.",
        };
        Line::from(Span::styled(
            empty_msg,
            Style::default().fg(Color::DarkGray),
        ))
    };

    let line2 = if let Some(child) = selected_opt {
        Line::from(vec![
            Span::styled(lang.path_label(), Style::default().fg(Color::DarkGray)),
            Span::styled(
                child.path.to_string_lossy(),
                Style::default().fg(Color::White),
            ),
        ])
    } else {
        Line::raw("")
    };

    let mut hud_lines = vec![line1];
    if let Some(ref q) = app.search_query {
        let (prompt_text, hint_text) = match lang {
            crate::i18n::Language::Vi => ("TÌM KIẾM SPOTLIGHT", "Enter: Giữ vị trí | Esc: Hủy"),
            crate::i18n::Language::En => ("SPOTLIGHT SEARCH", "Enter: Confirm | Esc: Cancel"),
        };
        hud_lines.push(Line::from(vec![
            Span::styled(
                format!(" [{}] /", prompt_text),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}_", q),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({})", hint_text),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        hud_lines.push(line2);
    }

    let is_protected = selected_opt.map(|c| c.is_system_protected).unwrap_or(false);
    let mut key_spans = vec![
        Span::styled("[←/→/h/l]", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" {}  ", lang.nav_stroll())),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" {}  ", lang.nav_enter())),
        Span::styled("[Esc/Backspace]", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" {}  ", lang.nav_back())),
        Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" {}  ", lang.nav_sort())),
    ];

    if is_protected {
        key_spans.push(Span::styled(
            "[d/Del]",
            Style::default().fg(Color::DarkGray),
        ));
        key_spans.push(Span::styled(
            format!(" {}  ", lang.protected_demolish_disabled_key()),
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        key_spans.push(Span::styled("[d/Del]", Style::default().fg(Color::Red)));
        key_spans.push(Span::raw(format!(" {}  ", lang.nav_demolish())));
    }

    key_spans.push(Span::styled("[o]", Style::default().fg(Color::LightGreen)));
    key_spans.push(Span::raw(format!(
        " {}  ",
        if lang == crate::i18n::Language::Vi {
            "Mở"
        } else {
            "Open"
        }
    )));
    key_spans.push(Span::styled("[/]", Style::default().fg(Color::LightYellow)));
    key_spans.push(Span::raw(format!(
        " {}  ",
        if lang == crate::i18n::Language::Vi {
            "Tìm"
        } else {
            "Search"
        }
    )));
    if app.is_in_scrapyard() {
        key_spans.push(Span::styled("[c]", Style::default().fg(Color::LightRed)));
        key_spans.push(Span::raw(format!(
            " {}  ",
            if lang == crate::i18n::Language::Vi {
                "Xe ủi Dọn Tất Cả"
            } else {
                "Bulldozer Clean All"
            }
        )));
    } else {
        key_spans.push(Span::styled(
            "[J]",
            Style::default().fg(Color::LightMagenta),
        ));
        key_spans.push(Span::raw(format!(
            " {}  ",
            if lang == crate::i18n::Language::Vi {
                "Phế liệu"
            } else {
                "Scrapyard"
            }
        )));
    }
    key_spans.push(Span::styled("[r]", Style::default().fg(Color::Cyan)));
    key_spans.push(Span::raw(format!(" {}  ", lang.nav_rescan())));
    key_spans.push(Span::styled("[L]", Style::default().fg(Color::Green)));
    key_spans.push(Span::raw(format!(
        " {} ({})  ",
        lang.nav_lang(),
        lang.display_name()
    )));
    key_spans.push(Span::styled("[q]", Style::default().fg(Color::Magenta)));
    key_spans.push(Span::raw(format!(" {}", lang.nav_quit())));

    let line3 = Line::from(key_spans);
    hud_lines.push(line3);
    let paragraph = Paragraph::new(hud_lines).block(block);
    frame.render_widget(paragraph, area);
}
