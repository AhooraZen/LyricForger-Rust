use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, BorderType, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Wrap,
    },
    Frame,
};

use crate::types::{ActiveInput, AppState, Screen};

pub struct UI;

impl UI {
    pub fn draw(f: &mut Frame, state: &AppState) {
        let area = f.area();

        // Ensure minimal area sanity check
        if area.width < 10 || area.height < 5 {
            let p = Paragraph::new("Terminal too small! Please enlarge.").alignment(Alignment::Center);
            f.render_widget(p, area);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Top Header
                Constraint::Min(1),    // Main Content
                Constraint::Length(2), // Bottom Key Hints
            ])
            .split(area);

        Self::draw_header(f, state, chunks[0]);

        match state.current_screen {
            Screen::Setup => Self::draw_setup_screen(f, state, chunks[1]),
            Screen::Analysis => Self::draw_analysis_screen(f, state, chunks[1]),
            Screen::Forging => Self::draw_forging_screen(f, state, chunks[1]),
            Screen::Summary => Self::draw_summary_screen(f, state, chunks[1]),
        }

        Self::draw_footer(f, state, chunks[2]);

        if state.show_help_modal {
            Self::draw_help_modal(f, area);
        }

        if state.show_file_picker {
            Self::draw_file_picker_modal(f, state, area);
        }
    }

    fn draw_header(f: &mut Frame, state: &AppState, area: Rect) {
        let step_setup = if state.current_screen == Screen::Setup {
            Span::styled(" 1. SETUP ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 1. Setup ", Style::default().fg(Color::Gray))
        };

        let step_analysis = if state.current_screen == Screen::Analysis {
            Span::styled(" 2. MATCH ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 2. Match ", Style::default().fg(Color::Gray))
        };

        let step_forging = if state.current_screen == Screen::Forging {
            Span::styled(" 3. FORGE ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 3. Forge ", Style::default().fg(Color::Gray))
        };

        let step_summary = if state.current_screen == Screen::Summary {
            Span::styled(" 4. DONE ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 4. Done ", Style::default().fg(Color::Gray))
        };

        let title = vec![
            Span::styled(" ⚡ LYRIC FORGER ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            step_setup,
            Span::raw("➔"),
            step_analysis,
            Span::raw("➔"),
            step_forging,
            Span::raw("➔"),
            step_summary,
        ];

        let header = Paragraph::new(Line::from(title))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Left);

        f.render_widget(header, area);
    }

    fn draw_footer(f: &mut Frame, state: &AppState, area: Rect) {
        let hints = match state.current_screen {
            Screen::Setup => " [Tab] Next Field  [Ctrl+P] Browse Files  [Enter] Start Scan  [F1] Help  [Esc] Quit ",
            Screen::Analysis => " [↑/↓] Select Pair  [Space] Filter Unmatched  [Enter] FORGE  [Esc] Back ",
            Screen::Forging => " ⚡ Forging lyrics into audio metadata... Please wait ",
            Screen::Summary => " [Enter] New Scan Job  [Esc] Exit ",
        };

        let footer = Paragraph::new(Span::styled(hints, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .alignment(Alignment::Center);

        f.render_widget(footer, area);
    }

    fn draw_setup_screen(f: &mut Frame, state: &AppState, area: Rect) {
        // Use flexible proportions for responsiveness on small screens
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),          // Input fields container
                Constraint::Percentage(40),  // Info box
            ])
            .margin(1)
            .split(area);

        let input_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(2),
                Constraint::Min(2),
                Constraint::Min(2),
                Constraint::Min(2),
            ])
            .split(chunks[0]);

        let active_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        let inactive_style = Style::default().fg(Color::Gray);

        let music_style = if state.active_input == ActiveInput::MusicPath { active_style } else { inactive_style };
        let lyrics_style = if state.active_input == ActiveInput::LyricsPath { active_style } else { inactive_style };
        let output_style = if state.active_input == ActiveInput::OutputPath { active_style } else { inactive_style };
        let thresh_style = if state.active_input == ActiveInput::Threshold { active_style } else { inactive_style };

        let m_str = if state.music_path_input.is_empty() { "(Empty - Press Ctrl+P to browse zip/folder)" } else { &state.music_path_input };
        let l_str = if state.lyrics_path_input.is_empty() { "(Empty - Press Ctrl+P to browse zip/folder)" } else { &state.lyrics_path_input };

        let p_music = Paragraph::new(format!(" 🎵 Music ZIP / Folder:  {}\n   [Ctrl+P to browse archive/folder]", m_str)).style(music_style);
        let p_lyrics = Paragraph::new(format!(" 📄 Lyrics ZIP / Folder: {}\n   [Ctrl+P to browse archive/folder]", l_str)).style(lyrics_style);
        let p_output = Paragraph::new(format!(" 📁 Output Folder:      {}", state.output_path_input)).style(output_style);
        let p_thresh = Paragraph::new(format!(" 🎯 Match Threshold (%): {}%", state.threshold)).style(thresh_style);

        f.render_widget(p_music, input_chunks[0]);
        f.render_widget(p_lyrics, input_chunks[1]);
        f.render_widget(p_output, input_chunks[2]);
        f.render_widget(p_thresh, input_chunks[3]);

        let mut info_text = vec![
            Line::from(Span::styled("✨ 3 Smart Matching Strategies Enabled:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from("  1. 🎯 Exact Clean: Strips track #, release noise, brackets & punctuation."),
            Line::from("  2. 🏷️ Metadata Header: Compares ID3/Vorbis tags with [ti:] & [ar:] LRC headers."),
            Line::from("  3. 🔤 Fuzzy Distance: Levenshtein & Token ratio for minor spelling differences."),
            Line::from(""),
            Line::from(Span::styled("📱 Samsung Music Spec Support:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from("  • MP3 USLT ID3v2 frames & FLAC Vorbis LYRICS comments."),
        ];

        if let Some(ref err) = state.error_msg {
            info_text.push(Line::from(""));
            info_text.push(Line::from(Span::styled(format!("❌ Error: {}", err), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))));
        }

        let info_box = Paragraph::new(info_text)
            .block(
                Block::default()
                    .title(" Smart Forger Engine ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(info_box, chunks[1]);
    }

    fn draw_analysis_screen(f: &mut Frame, state: &AppState, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Table list
                Constraint::Percentage(40), // Details & Preview
            ])
            .margin(1)
            .split(area);

        let displayed_matches: Vec<(usize, &crate::types::MatchResult)> = state
            .matches
            .iter()
            .enumerate()
            .filter(|(_, m)| !state.filter_unmatched_only || m.lyric_id.is_none())
            .collect();

        let rows: Vec<Row> = displayed_matches
            .iter()
            .map(|(idx, m)| {
                let music_name = state
                    .music_files
                    .iter()
                    .find(|f| f.id == m.music_id)
                    .map(|f| f.filename.as_str())
                    .unwrap_or("Unknown");

                let (lyric_name, style, badge) = if let Some(lid) = m.lyric_id {
                    let lname = state
                        .lyric_files
                        .iter()
                        .find(|f| f.id == lid)
                        .map(|f| f.filename.as_str())
                        .unwrap_or("Unknown");

                    if m.confidence >= 80 {
                        (lname, Style::default().fg(Color::Green), format!("🟢 {}%", m.confidence))
                    } else {
                        (lname, Style::default().fg(Color::Yellow), format!("🟡 {}%", m.confidence))
                    }
                } else {
                    ("❌ No match found", Style::default().fg(Color::Red), String::from("🔴 0%"))
                };

                let strategy_str = m
                    .strategy
                    .as_ref()
                    .map(|s| s.label())
                    .unwrap_or("NONE");

                let is_selected = *idx == state.selected_match_idx;
                let row_style = if is_selected {
                    style.bg(Color::DarkGray).add_modifier(Modifier::BOLD)
                } else {
                    style
                };

                Row::new(vec![
                    format!("{}", idx + 1),
                    music_name.to_string(),
                    lyric_name.to_string(),
                    badge,
                    strategy_str.to_string(),
                ])
                .style(row_style)
            })
            .collect();

        let header_cells = ["#", "Music File", "Matched Lyrics", "Conf", "Strategy"]
            .iter()
            .map(|h| Span::styled(*h, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));

        let header = Row::new(header_cells).style(Style::default().bg(Color::Blue)).height(1);

        let table_title = format!(
            " Matches ({}/{} matched) ",
            state.matches.iter().filter(|m| m.lyric_id.is_some()).count(),
            state.matches.len()
        );

        let table = Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Length(7),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(table_title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(table, chunks[0]);

        if let Some(m) = state.matches.get(state.selected_match_idx) {
            let music = state.music_files.iter().find(|f| f.id == m.music_id);
            let lyric = m.lyric_id.and_then(|lid| state.lyric_files.iter().find(|f| f.id == lid));

            let mut preview_lines = vec![
                Line::from(Span::styled("🎵 Audio Details:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from(format!(" File: {}", music.map(|f| f.filename.as_str()).unwrap_or("-"))),
                Line::from(format!(" Title: {}", music.and_then(|f| f.title.as_deref()).unwrap_or("N/A"))),
                Line::from(format!(" Artist: {}", music.and_then(|f| f.artist.as_deref()).unwrap_or("N/A"))),
                Line::from(""),
                Line::from(Span::styled("📄 Lyrics Details:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
                Line::from(format!(" File: {}", lyric.map(|f| f.filename.as_str()).unwrap_or("None"))),
                Line::from(""),
                Line::from(Span::styled("📜 LRC Snippet:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            ];

            if let Some(l) = lyric {
                for line in l.content.lines().take(6) {
                    preview_lines.push(Line::from(format!("  {}", line)));
                }
            } else {
                preview_lines.push(Line::from(Span::styled("  (No lyrics file paired)", Style::default().fg(Color::DarkGray))));
            }

            let preview_box = Paragraph::new(preview_lines)
                .block(
                    Block::default()
                        .title(" Preview ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Magenta)),
                )
                .wrap(Wrap { trim: true });

            f.render_widget(preview_box, chunks[1]);
        }
    }

    fn draw_forging_screen(f: &mut Frame, state: &AppState, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(1),
            ])
            .margin(1)
            .split(area);

        let total = state.matches.len();
        let current = state.processed_count;
        let percent = if total > 0 {
            ((current as f64 / total as f64) * 100.0) as u16
        } else {
            0
        };

        let label = format!("Forging Lyrics: {}/{} ({}%)", current, total, percent);

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(" Forging Engine Progress ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::BOLD))
            .percent(percent)
            .label(label);

        f.render_widget(gauge, chunks[0]);

        let log_items: Vec<ListItem> = state
            .logs
            .iter()
            .rev()
            .take(12)
            .map(|log| ListItem::new(Span::raw(log.clone())))
            .collect();

        let logs_list = List::new(log_items).block(
            Block::default()
                .title(" Live Logs ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(logs_list, chunks[1]);
    }

    fn draw_summary_screen(f: &mut Frame, state: &AppState, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),
                Constraint::Percentage(50),
            ])
            .margin(1)
            .split(area);

        let summary_text = vec![
            Line::from(Span::styled("🎉 FORGING COMPLETED!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("  ✅ Success: {}  ", state.success_count), Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(format!("  ⚠️ Skipped: {}  ", state.fail_count), Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
        ];

        let summary_box = Paragraph::new(summary_text).block(
            Block::default()
                .title(" Summary ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        );

        f.render_widget(summary_box, chunks[0]);

        let tips_text = vec![
            Line::from(Span::styled("💡 View lyrics in Samsung Music:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from("  1. Clear Samsung Music app cache or restart phone."),
            Line::from("  2. Tap song cover art during playback to expand lyrics!"),
            Line::from(""),
            Line::from(Span::styled("Press [ENTER] to process another zip file or [ESC] to exit.", Style::default().fg(Color::Cyan))),
        ];

        let tips_box = Paragraph::new(tips_text).block(
            Block::default()
                .title(" Samsung Music Guide ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(tips_box, chunks[1]);
    }

    fn draw_help_modal(f: &mut Frame, area: Rect) {
        let popup_area = Rect {
            x: area.width / 8,
            y: area.height / 8,
            width: (area.width * 3) / 4,
            height: (area.height * 3) / 4,
        };

        f.render_widget(Clear, popup_area);

        let help_text = vec![
            Line::from(Span::styled("⚡ LYRIC FORGER HELP & KEYBINDINGS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Controls:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from("  • Tab           : Cycle input fields"),
            Line::from("  • Ctrl+P        : Open Archive & Folder Picker Browser"),
            Line::from("  • Ctrl+U        : Clear current text field"),
            Line::from("  • ↑ / ↓ Arrows  : Scroll match table / navigate file picker"),
            Line::from("  • Enter         : Select file/folder or start process"),
            Line::from("  • Esc           : Back to previous screen / close modal"),
            Line::from(""),
            Line::from(Span::styled("Press [Esc] or [F1] to close.", Style::default().fg(Color::Magenta))),
        ];

        let modal = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" User Manual ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(modal, popup_area);
    }

    fn draw_file_picker_modal(f: &mut Frame, state: &AppState, area: Rect) {
        let popup_area = Rect {
            x: area.width / 10,
            y: area.height / 10,
            width: (area.width * 4) / 5,
            height: (area.height * 4) / 5,
        };

        f.render_widget(Clear, popup_area);

        let picker = &state.file_picker;
        let dir_str = picker.current_dir.to_string_lossy();

        let items: Vec<ListItem> = picker
            .entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let is_sel = idx == picker.selected_idx;
                let icon = if entry.name == ".." {
                    "📁 [UP]"
                } else if entry.is_dir {
                    "📂 [DIR]"
                } else if entry.is_archive {
                    "📦 [ARCHIVE]"
                } else {
                    "📄 [FILE]"
                };

                let style = if is_sel {
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else if entry.is_dir {
                    Style::default().fg(Color::Yellow)
                } else if entry.is_archive {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Span::styled(format!(" {}  {}", icon, entry.name), style))
            })
            .collect();

        let title_str = format!(" Archive & Folder Picker ➔ {} ", dir_str);

        let list_widget = List::new(items).block(
            Block::default()
                .title(title_str)
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(list_widget, popup_area);
    }
}
