mod matcher;
mod tagger;
mod types;
mod ui;

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use walkdir::WalkDir;
use zip::ZipArchive;

use matcher::MatcherEngine;
use tagger::TaggerEngine;
use types::{ActiveInput, AppState, LyricFile, MusicFile, Screen};
use ui::UI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::default();

    // Auto-detect sample zip files if they exist in cwd
    if Path::new("sample_music.zip").exists() {
        state.music_path_input = String::from("sample_music.zip");
    }
    if Path::new("sample_lyrics.zip").exists() {
        state.lyrics_path_input = String::from("sample_lyrics.zip");
    }

    // Run application loop
    let res = run_app(&mut terminal, &mut state);

    // Restore terminal state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Cleanup temp dir if created
    if let Some(ref temp_path) = state.temp_dir {
        let _ = fs::remove_dir_all(temp_path);
    }

    if let Err(err) = res {
        println!("Lyric Forger Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| UI::draw(f, state))?;

        if state.current_screen == Screen::Forging && state.is_processing {
            execute_forging_step(state);
            if state.processed_count >= state.matches.len() {
                state.is_processing = false;
                state.current_screen = Screen::Summary;
            }
            continue;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Toggle help modal globally
                if key.code == KeyCode::Char('h') || key.code == KeyCode::Char('?') {
                    state.show_help_modal = !state.show_help_modal;
                    continue;
                }

                if state.show_help_modal {
                    if key.code == KeyCode::Esc || key.code == KeyCode::Char('h') {
                        state.show_help_modal = false;
                    }
                    continue;
                }

                match state.current_screen {
                    Screen::Setup => handle_setup_input(key.code, state),
                    Screen::Analysis => handle_analysis_input(key.code, state),
                    Screen::Forging => {}
                    Screen::Summary => handle_summary_input(key.code, state),
                }

                // Global quit command: Ctrl+C
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(());
                }
            }
        }
    }
}

fn handle_setup_input(code: KeyCode, state: &mut AppState) {
    match code {
        KeyCode::Esc => {
            std::process::exit(0);
        }
        KeyCode::Tab => {
            state.active_input = match state.active_input {
                ActiveInput::MusicPath => ActiveInput::LyricsPath,
                ActiveInput::LyricsPath => ActiveInput::OutputPath,
                ActiveInput::OutputPath => ActiveInput::Threshold,
                ActiveInput::Threshold => ActiveInput::MusicPath,
            };
        }
        KeyCode::BackTab => {
            state.active_input = match state.active_input {
                ActiveInput::MusicPath => ActiveInput::Threshold,
                ActiveInput::LyricsPath => ActiveInput::MusicPath,
                ActiveInput::OutputPath => ActiveInput::LyricsPath,
                ActiveInput::Threshold => ActiveInput::OutputPath,
            };
        }
        KeyCode::Char(c) => {
            match state.active_input {
                ActiveInput::MusicPath => state.music_path_input.push(c),
                ActiveInput::LyricsPath => state.lyrics_path_input.push(c),
                ActiveInput::OutputPath => state.output_path_input.push(c),
                ActiveInput::Threshold => {
                    if c.is_ascii_digit() {
                        let val = format!("{}{}", state.threshold, c);
                        if let Ok(num) = val.parse::<u32>() {
                            state.threshold = num.min(100);
                        }
                    }
                }
            }
        }
        KeyCode::Backspace => {
            match state.active_input {
                ActiveInput::MusicPath => { state.music_path_input.pop(); }
                ActiveInput::LyricsPath => { state.lyrics_path_input.pop(); }
                ActiveInput::OutputPath => { state.output_path_input.pop(); }
                ActiveInput::Threshold => {
                    state.threshold /= 10;
                }
            }
        }
        KeyCode::Enter => {
            // Perform Scan & Match
            if perform_scan_and_match(state) {
                state.current_screen = Screen::Analysis;
            }
        }
        _ => {}
    }
}

fn handle_analysis_input(code: KeyCode, state: &mut AppState) {
    match code {
        KeyCode::Esc => {
            state.current_screen = Screen::Setup;
        }
        KeyCode::Up => {
            if state.selected_match_idx > 0 {
                state.selected_match_idx -= 1;
            }
        }
        KeyCode::Down => {
            if state.selected_match_idx + 1 < state.matches.len() {
                state.selected_match_idx += 1;
            }
        }
        KeyCode::Char(' ') => {
            state.filter_unmatched_only = !state.filter_unmatched_only;
        }
        KeyCode::Enter => {
            // Start Forging Process
            state.current_screen = Screen::Forging;
            state.is_processing = true;
            state.processed_count = 0;
            state.success_count = 0;
            state.fail_count = 0;
            state.logs.clear();
            state.logs.push(String::from("🚀 Initializing Forging Engine..."));
        }
        _ => {}
    }
}

fn handle_summary_input(code: KeyCode, state: &mut AppState) {
    match code {
        KeyCode::Enter => {
            state.current_screen = Screen::Setup;
        }
        KeyCode::Esc => {
            std::process::exit(0);
        }
        _ => {}
    }
}

fn perform_scan_and_match(state: &mut AppState) -> bool {
    state.music_files.clear();
    state.lyric_files.clear();
    state.matches.clear();
    state.selected_match_idx = 0;

    let temp_root = std::env::temp_dir().join("lyric_forger_work");
    let _ = fs::create_dir_all(&temp_root);
    state.temp_dir = Some(temp_root.clone());

    let music_dir = process_input_target(&state.music_path_input, &temp_root.join("music"));
    let lyrics_dir = process_input_target(&state.lyrics_path_input, &temp_root.join("lyrics"));

    if let Some(ref mdir) = music_dir {
        let mut id_counter = 0;
        for entry in WalkDir::new(mdir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if ["mp3", "flac", "m4a", "ogg"].contains(&ext.as_str()) {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    let clean_name = MatcherEngine::clean_string(&filename);
                    let (title, artist) = TaggerEngine::read_audio_tags(path);

                    state.music_files.push(MusicFile {
                        id: id_counter,
                        path: path.to_path_buf(),
                        filename,
                        clean_name,
                        title,
                        artist,
                    });
                    id_counter += 1;
                }
            }
        }
    }

    if let Some(ref ldir) = lyrics_dir {
        let mut id_counter = 0;
        for entry in WalkDir::new(ldir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if ["lrc", "txt"].contains(&ext.as_str()) {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    let clean_name = MatcherEngine::clean_string(&filename);
                    let content = fs::read_to_string(path).unwrap_or_default();
                    let (title_header, artist_header) = TaggerEngine::parse_lrc_headers(&content);

                    state.lyric_files.push(LyricFile {
                        id: id_counter,
                        path: path.to_path_buf(),
                        filename,
                        clean_name,
                        title_header,
                        artist_header,
                        content,
                    });
                    id_counter += 1;
                }
            }
        }
    }

    if state.music_files.is_empty() {
        return false;
    }

    state.matches = MatcherEngine::find_best_matches(
        &state.music_files,
        &state.lyric_files,
        state.threshold,
    );

    true
}

fn process_input_target(input_path_str: &str, extract_dest: &Path) -> Option<PathBuf> {
    let p = Path::new(input_path_str);
    if !p.exists() {
        return None;
    }

    if p.is_dir() {
        return Some(p.to_path_buf());
    }

    if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("zip") {
        if let Ok(file) = File::open(p) {
            if let Ok(mut archive) = ZipArchive::new(file) {
                let _ = fs::create_dir_all(extract_dest);
                for i in 0..archive.len() {
                    if let Ok(mut file) = archive.by_index(i) {
                        let outpath = match file.enclosed_name() {
                            Some(path) => extract_dest.join(path),
                            None => continue,
                        };
                        if file.name().ends_with('/') {
                            let _ = fs::create_dir_all(&outpath);
                        } else {
                            if let Some(p) = outpath.parent() {
                                if !p.exists() {
                                    let _ = fs::create_dir_all(p);
                                }
                            }
                            if let Ok(mut outfile) = File::create(&outpath) {
                                let _ = io::copy(&mut file, &mut outfile);
                            }
                        }
                    }
                }
                return Some(extract_dest.to_path_buf());
            }
        }
    }

    None
}

fn execute_forging_step(state: &mut AppState) {
    if state.processed_count >= state.matches.len() {
        return;
    }

    let idx = state.processed_count;
    let match_item = &state.matches[idx];

    let music = state.music_files.iter().find(|f| f.id == match_item.music_id).cloned();
    let lyric = match_item.lyric_id.and_then(|lid| state.lyric_files.iter().find(|f| f.id == lid)).cloned();

    let m_filename = music.as_ref().map(|m| m.filename.clone()).unwrap_or_default();

    if let (Some(m), Some(l)) = (music, lyric) {
        match TaggerEngine::embed_lyrics(&m.path, &l.content) {
            Ok(_) => {
                state.success_count += 1;
                state.logs.push(format!("✅ Forged lyrics into: '{}' (via {})", m.filename, l.filename));
            }
            Err(e) => {
                state.fail_count += 1;
                state.logs.push(format!("❌ Error embedding into '{}': {}", m.filename, e));
            }
        }
    } else {
        state.fail_count += 1;
        if !m_filename.is_empty() {
            state.logs.push(format!("⚠️ Skipped '{}' - No matching lyrics file found.", m_filename));
        }
    }

    state.processed_count += 1;
}
