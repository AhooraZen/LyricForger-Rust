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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header bar
                Constraint::Min(0),    // Main content area
                Constraint::Length(2), // Footer key hints
            ])
            .split(f.area());

        Self::draw_header(f, state, chunks[0]);

        match state.current_screen {
            Screen::Setup => Self::draw_setup_screen(f, state, chunks[1]),
            Screen::Analysis => Self::draw_analysis_screen(f, state, chunks[1]),
            Screen::Forging => Self::draw_forging_screen(f, state, chunks[1]),
            Screen::Summary => Self::draw_summary_screen(f, state, chunks[1]),
        }

        Self::draw_footer(f, state, chunks[2]);

        if state.show_help_modal {
            Self::draw_help_modal(f, f.area());
        }
    }

    fn draw_header(f: &mut Frame, state: &AppState, area: Rect) {
        let step_setup = if state.current_screen == Screen::Setup {
            Span::styled(" 1. SETUP ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 1. Setup ", Style::default().fg(Color::Gray))
        };

        let step_analysis = if state.current_screen == Screen::Analysis {
            Span::styled(" 2. MATCH ANALYSIS ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 2. Analysis ", Style::default().fg(Color::Gray))
        };

        let step_forging = if state.current_screen == Screen::Forging {
            Span::styled(" 3. FORGING ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 3. Forging ", Style::default().fg(Color::Gray))
        };

        let step_summary = if state.current_screen == Screen::Summary {
            Span::styled(" 4. DONE ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 4. Summary ", Style::default().fg(Color::Gray))
        };

        let title = vec![
            Span::styled(" ⚡ LYRIC FORGER ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("│ Termux & Android │ ", Style::default().fg(Color::DarkGray)),
            step_setup,
            Span::raw(" ➔ "),
            step_analysis,
            Span::raw(" ➔ "),
            step_forging,
            Span::raw(" ➔ "),
            step_summary,
            Span::raw("   "),
            Span::styled("[H] Help", Style::default().fg(Color::Yellow)),
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
            Screen::Setup => " [Tab] Switch Field  [Type] Enter Path  [Enter] Start Scan  [H] Help Modal  [Esc] Quit ",
            Screen::Analysis => " [↑/↓] Navigate  [Space] Filter Unmatched  [Enter] FORGE LYRICS  [H] Help  [Esc] Back ",
            Screen::Forging => " [Processing...] Embedding lyrics into music metadata... ",
            Screen::Summary => " [Enter] New Processing Job  [Esc] Exit ",
        };

        let footer = Paragraph::new(Span::styled(hints, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .alignment(Alignment::Center);

        f.render_widget(footer, area);
    }

    fn draw_setup_screen(f: &mut Frame, state: &AppState, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Inputs
                Constraint::Min(0),     // Info & strategy explanation panel
            ])
            .margin(1)
            .split(area);

        let input_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
            ])
            .split(chunks[0]);

        let active_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        let inactive_style = Style::default().fg(Color::Gray);

        let music_style = if state.active_input == ActiveInput::MusicPath { active_style } else { inactive_style };
        let lyrics_style = if state.active_input == ActiveInput::LyricsPath { active_style } else { inactive_style };
        let output_style = if state.active_input == ActiveInput::OutputPath { active_style } else { inactive_style };
        let thresh_style = if state.active_input == ActiveInput::Threshold { active_style } else { inactive_style };

        let p_music = Paragraph::new(format!(" Music ZIP or Folder:  {}", state.music_path_input))
            .style(music_style);
        let p_lyrics = Paragraph::new(format!(" Lyrics ZIP or Folder: {}", state.lyrics_path_input))
            .style(lyrics_style);
        let p_output = Paragraph::new(format!(" Output Destination:  {}", state.output_path_input))
            .style(output_style);
        let p_thresh = Paragraph::new(format!(" Match Threshold (%):  {}%", state.threshold))
            .style(thresh_style);

        f.render_widget(p_music, input_chunks[0]);
        f.render_widget(p_lyrics, input_chunks[1]);
        f.render_widget(p_output, input_chunks[2]);
        f.render_widget(p_thresh, input_chunks[3]);

        let info_text = vec![
            Line::from(Span::styled("✨ 3 Smart Matching Strategies Enabled:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from("  1. 🎯 Exact Clean Match: Normalizes filenames, strips track numbers, brackets & keywords."),
            Line::from("  2. 🏷️ ID3 / Vorbis Metadata Tag Match: Compares internal audio tags with [ti:] and [ar:] LRC headers."),
            Line::from("  3. 🔤 Fuzzy Distance Ratio: Uses Levenshtein / Sorensen-Dice distance for slight typos or artist variations."),
            Line::from(""),
            Line::from(Span::styled("📱 Samsung Music & Android Player Support:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from("  • MP3 files receive standard ID3v2 USLT (Unsynchronized Lyrics) frame."),
            Line::from("  • FLAC files receive Vorbis LYRICS and UNSYNCEDLYRICS comment tags."),
            Line::from(""),
            Line::from(Span::styled("Press [ENTER] to begin scanning zip files and generating smart matches!", Style::default().fg(Color::Yellow))),
        ];

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
                Constraint::Percentage(65), // Table list
                Constraint::Percentage(35), // Details & Preview
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

        let header_cells = ["#", "Music File", "Matched Lyrics File", "Conf", "Strategy"]
            .iter()
            .map(|h| Span::styled(*h, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));

        let header = Row::new(header_cells).style(Style::default().bg(Color::Blue)).height(1);

        let table_title = format!(
            " Matches ({}/{} matched) {} ",
            state.matches.iter().filter(|m| m.lyric_id.is_some()).count(),
            state.matches.len(),
            if state.filter_unmatched_only { "[Filtered: Unmatched]" } else { "" }
        );

        let table = Table::new(
            rows,
            [
                Constraint::Length(4),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Length(8),
                Constraint::Length(14),
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
                Line::from(format!(" Title Tag: {}", music.and_then(|f| f.title.as_deref()).unwrap_or("N/A"))),
                Line::from(format!(" Artist Tag: {}", music.and_then(|f| f.artist.as_deref()).unwrap_or("N/A"))),
                Line::from(""),
                Line::from(Span::styled("📄 Lyrics Details:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
                Line::from(format!(" File: {}", lyric.map(|f| f.filename.as_str()).unwrap_or("None"))),
                Line::from(format!(" Title Tag: {}", lyric.and_then(|f| f.title_header.as_deref()).unwrap_or("N/A"))),
                Line::from(format!(" Artist Tag: {}", lyric.and_then(|f| f.artist_header.as_deref()).unwrap_or("N/A"))),
                Line::from(""),
                Line::from(Span::styled("🔍 Strategy Match Info:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(format!(" Confidence: {}%", m.confidence)),
                Line::from(format!(" Details: {}", m.detail)),
                Line::from(""),
                Line::from(Span::styled("📜 LRC Content Snippet:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            ];

            if let Some(l) = lyric {
                for line in l.content.lines().take(8) {
                    preview_lines.push(Line::from(format!("  {}", line)));
                }
            } else {
                preview_lines.push(Line::from(Span::styled("  (No lyrics content to display)", Style::default().fg(Color::DarkGray))));
            }

            let preview_box = Paragraph::new(preview_lines)
                .block(
                    Block::default()
                        .title(" Match Preview ")
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
                Constraint::Length(5), // Progress bar
                Constraint::Min(0),    // Log stream
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
            .take(15)
            .map(|log| ListItem::new(Span::raw(log.clone())))
            .collect();

        let logs_list = List::new(log_items).block(
            Block::default()
                .title(" Live Forging Logs ")
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
                Constraint::Length(7), // Cards summary
                Constraint::Min(0),    // Samsung Music tips
            ])
            .margin(1)
            .split(area);

        let summary_text = vec![
            Line::from(Span::styled("🎉 FORGING COMPLETED SUCCESSFULLY!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("  ✅ Successfully Embedded: {}  ", state.success_count), Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("   "),
                Span::styled(format!("  ⚠️ Skipped / Unmatched: {}  ", state.fail_count), Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("   "),
                Span::styled(format!("  📁 Total Songs Processed: {}  ", state.processed_count), Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
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
            Line::from(Span::styled("💡 How to view your lyrics in Samsung Music & Android Players:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  1. 🔄 Refresh Media Scanner: Android caches music metadata. If lyrics don't show immediately,"),
            Line::from("     go to Samsung Music Settings ➔ Clear Cache or restart your device."),
            Line::from("  2. 🎶 Tap on Song Cover/Title: In Samsung Music, tap on the song title / album art view while playing"),
            Line::from("     to expand the synchronized / unsynchronized lyrics panel!"),
            Line::from("  3. 🎧 Works with Musicolet, Poweramp, Retro Music Player, and native Android media framework."),
            Line::from(""),
            Line::from(Span::styled("Press [ENTER] to process another zip file or [ESC] to exit.", Style::default().fg(Color::Cyan))),
        ];

        let tips_box = Paragraph::new(tips_text).block(
            Block::default()
                .title(" Android & Samsung Music Guide ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(tips_box, chunks[1]);
    }

    fn draw_help_modal(f: &mut Frame, area: Rect) {
        let popup_area = Rect {
            x: area.width / 6,
            y: area.height / 6,
            width: (area.width * 2) / 3,
            height: (area.height * 2) / 3,
        };

        f.render_widget(Clear, popup_area);

        let help_text = vec![
            Line::from(Span::styled("⚡ LYRIC FORGER HELP & KEYBINDINGS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Keyboard Controls:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from("  • Tab / Shift+Tab : Switch active input field on Setup Screen"),
            Line::from("  • ↑ / ↓ Arrows    : Navigate table of matches on Analysis Screen"),
            Line::from("  • Spacebar        : Toggle filter to show unmatched songs only"),
            Line::from("  • Enter           : Confirm action / Start Scan / Start Forging"),
            Line::from("  • H or ?          : Toggle this Help popup dialog"),
            Line::from("  • Esc / Ctrl+C    : Back or Exit"),
            Line::from(""),
            Line::from(Span::styled("Samsung Music & Android Integration:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from("  • USLT ID3v2 frames are injected into MP3 files."),
            Line::from("  • Vorbis LYRICS comments are injected into FLAC files."),
            Line::from(""),
            Line::from(Span::styled("Press [H] or [Esc] to close this modal.", Style::default().fg(Color::Magenta))),
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
}
