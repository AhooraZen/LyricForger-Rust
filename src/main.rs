mod matcher;
mod tagger;
mod types;
mod ui;

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::{Position, Size}, Terminal};
use walkdir::WalkDir;
use zip::ZipArchive;

use matcher::MatcherEngine;
use tagger::TaggerEngine;
use types::{ActiveInput, AppState, FilePickerEntry, LyricFile, MusicFile, Screen};
use ui::UI;

/// Bulletproof TermuxBackend wrapper for Android / Termux compatibility.
/// Intercepts and squelches Android kernel/libc IO errors like OS Error 34 (`ERANGE` / "Math result not representable")
/// during ioctl(TIOCGWINSZ) and termios calls.
pub struct TermuxBackend<W: Write> {
    inner: CrosstermBackend<W>,
}

impl<W: Write> TermuxBackend<W> {
    pub fn new(writer: W) -> Self {
        Self {
            inner: CrosstermBackend::new(writer),
        }
    }
}

impl<W: Write> ratatui::backend::Backend for TermuxBackend<W> {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
    {
        self.inner.draw(content).or_else(|_| Ok(()))
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.inner.hide_cursor().or_else(|_| Ok(()))
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.inner.show_cursor().or_else(|_| Ok(()))
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        self.inner.get_cursor_position().or_else(|_| Ok(Position::new(0, 0)))
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        self.inner.set_cursor_position(position).or_else(|_| Ok(()))
    }

    fn clear(&mut self) -> io::Result<()> {
        self.inner.clear().or_else(|_| Ok(()))
    }

    fn size(&self) -> io::Result<Size> {
        match self.inner.size() {
            Ok(size) => Ok(size),
            Err(_) => {
                let cols = std::env::var("COLUMNS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(80);
                let lines = std::env::var("LINES")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(24);
                Ok(Size::new(cols, lines))
            }
        }
    }

    fn window_size(&mut self) -> io::Result<ratatui::backend::WindowSize> {
        self.inner.window_size().or_else(|_| {
            let cols = std::env::var("COLUMNS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(80);
            let lines = std::env::var("LINES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(24);
            Ok(ratatui::backend::WindowSize {
                columns_rows: Size::new(cols, lines),
                pixels: Size::new(0, 0),
            })
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        ratatui::backend::Backend::flush(&mut self.inner).or_else(|_| Ok(()))
    }
}

fn main() {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        cleanup_terminal();
        default_panic(panic_info);
    }));

    if let Err(err) = run() {
        cleanup_terminal();
        eprintln!("Lyric Forger Run Error: {}", err);
    }
}

fn cleanup_terminal() {
    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::cursor::Show
    );
    let _ = stdout.flush();
}

fn run() -> io::Result<()> {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen, EnableMouseCapture);

    let backend = TermuxBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::default();

    if Path::new("sample_music.zip").exists() {
        state.music_path_input = String::from("sample_music.zip");
    }
    if Path::new("sample_lyrics.zip").exists() {
        state.lyrics_path_input = String::from("sample_lyrics.zip");
    }

    let res = run_app(&mut terminal, &mut state);

    cleanup_terminal();

    if let Some(ref temp_path) = state.temp_dir {
        let _ = fs::remove_dir_all(temp_path);
    }

    res
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
) -> io::Result<()> {
    loop {
        let _ = terminal.draw(|f| UI::draw(f, state));

        if state.current_screen == Screen::Forging && state.is_processing {
            execute_forging_step(state);
            if state.processed_count >= state.matches.len() {
                state.is_processing = false;
                state.current_screen = Screen::Summary;
            }
            continue;
        }

        let poll_res = event::poll(Duration::from_millis(100));
        if let Ok(true) = poll_res {
            if let Ok(ev) = event::read() {
                match ev {
                    Event::Key(key) => {
                        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            return Ok(());
                        }

                        if state.show_file_picker {
                            handle_file_picker_input(key.code, state);
                            continue;
                        }

                        if state.show_help_modal {
                            if key.code == KeyCode::Esc || key.code == KeyCode::F(1) {
                                state.show_help_modal = false;
                            }
                            continue;
                        }

                        if key.code == KeyCode::F(1) {
                            state.show_help_modal = true;
                            continue;
                        }

                        let should_quit = match state.current_screen {
                            Screen::Setup => handle_setup_input(key.code, key.modifiers, state),
                            Screen::Analysis => handle_analysis_input(key.code, state),
                            Screen::Forging => false,
                            Screen::Summary => handle_summary_input(key.code, state),
                        };

                        if should_quit {
                            return Ok(());
                        }
                    }

                    Event::Mouse(mouse) => {
                        if mouse.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                            handle_mouse_click(mouse.column, mouse.row, state);
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}

fn handle_mouse_click(_col: u16, row: u16, state: &mut AppState) {
    if state.show_file_picker || state.show_help_modal {
        return;
    }

    if state.current_screen == Screen::Setup {
        if (3..=5).contains(&row) {
            state.active_input = ActiveInput::MusicPath;
        } else if (6..=8).contains(&row) {
            state.active_input = ActiveInput::LyricsPath;
        } else if (9..=10).contains(&row) {
            state.active_input = ActiveInput::OutputPath;
        } else if (11..=12).contains(&row) {
            state.active_input = ActiveInput::Threshold;
        }
    }
}

fn handle_setup_input(code: KeyCode, modifiers: KeyModifiers, state: &mut AppState) -> bool {
    if code == KeyCode::Char('p') && modifiers.contains(KeyModifiers::CONTROL) {
        open_file_picker(state);
        return false;
    }

    if code == KeyCode::Char('u') && modifiers.contains(KeyModifiers::CONTROL) {
        match state.active_input {
            ActiveInput::MusicPath => state.music_path_input.clear(),
            ActiveInput::LyricsPath => state.lyrics_path_input.clear(),
            ActiveInput::OutputPath => state.output_path_input.clear(),
            ActiveInput::Threshold => state.threshold = 50,
        }
        return false;
    }

    match code {
        KeyCode::Esc => {
            return true;
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
            state.error_msg = None;
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
                ActiveInput::Threshold => { state.threshold /= 10; }
            }
        }
        KeyCode::Enter => {
            if perform_scan_and_match(state) {
                state.current_screen = Screen::Analysis;
            } else {
                state.error_msg = Some(String::from("No valid music tracks or archives found at specified path!"));
            }
        }
        _ => {}
    }

    false
}

fn handle_analysis_input(code: KeyCode, state: &mut AppState) -> bool {
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
    false
}

fn handle_summary_input(code: KeyCode, state: &mut AppState) -> bool {
    match code {
        KeyCode::Enter => {
            state.current_screen = Screen::Setup;
            false
        }
        KeyCode::Esc => true,
        _ => false,
    }
}

fn open_file_picker(state: &mut AppState) {
    state.file_picker.target_input = state.active_input.clone();
    
    let current_val = match state.active_input {
        ActiveInput::MusicPath => &state.music_path_input,
        ActiveInput::LyricsPath => &state.lyrics_path_input,
        ActiveInput::OutputPath => &state.output_path_input,
        ActiveInput::Threshold => "",
    };

    let path_val = Path::new(current_val);
    if path_val.exists() {
        if path_val.is_dir() {
            state.file_picker.current_dir = path_val.to_path_buf();
        } else if let Some(parent) = path_val.parent() {
            state.file_picker.current_dir = parent.to_path_buf();
        }
    } else {
        let termux_sd = Path::new("/storage/emulated/0");
        if termux_sd.exists() {
            state.file_picker.current_dir = termux_sd.to_path_buf();
        } else if let Ok(cwd) = std::env::current_dir() {
            state.file_picker.current_dir = cwd;
        }
    }

    load_file_picker_entries(state);
    state.show_file_picker = true;
}

fn load_file_picker_entries(state: &mut AppState) {
    state.file_picker.entries.clear();
    state.file_picker.selected_idx = 0;

    let current_dir = state.file_picker.current_dir.clone();

    if let Some(parent) = current_dir.parent() {
        state.file_picker.entries.push(FilePickerEntry {
            name: String::from(".."),
            path: parent.to_path_buf(),
            is_dir: true,
            is_archive: false,
        });
    }

    if let Ok(read_dir) = fs::read_dir(&current_dir) {
        let mut dirs = Vec::new();
        let mut archives = Vec::new();

        for entry_res in read_dir.flatten() {
            let path = entry_res.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();

            if name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                dirs.push(FilePickerEntry {
                    name,
                    path,
                    is_dir: true,
                    is_archive: false,
                });
            } else if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                let is_archive = ["zip", "rar", "7z", "tar", "gz"].contains(&ext.as_str());
                let is_audio_lyric = ["mp3", "flac", "m4a", "ogg", "lrc", "txt"].contains(&ext.as_str());

                if is_archive || is_audio_lyric {
                    archives.push(FilePickerEntry {
                        name,
                        path,
                        is_dir: false,
                        is_archive,
                    });
                }
            }
        }

        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        archives.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        state.file_picker.entries.extend(dirs);
        state.file_picker.entries.extend(archives);
    }
}

fn handle_file_picker_input(code: KeyCode, state: &mut AppState) {
    let entry_count = state.file_picker.entries.len();

    match code {
        KeyCode::Esc => {
            state.show_file_picker = false;
        }
        KeyCode::Up => {
            if state.file_picker.selected_idx > 0 {
                state.file_picker.selected_idx -= 1;
            }
        }
        KeyCode::Down => {
            if entry_count > 0 && state.file_picker.selected_idx + 1 < entry_count {
                state.file_picker.selected_idx += 1;
            }
        }
        KeyCode::Char(' ') => {
            let dir_path = state.file_picker.current_dir.to_string_lossy().to_string();
            apply_picker_selection(state, &dir_path);
            state.show_file_picker = false;
        }
        KeyCode::Enter => {
            if let Some(entry) = state.file_picker.entries.get(state.file_picker.selected_idx).cloned() {
                if entry.is_dir {
                    state.file_picker.current_dir = entry.path;
                    load_file_picker_entries(state);
                } else {
                    let path_str = entry.path.to_string_lossy().to_string();
                    apply_picker_selection(state, &path_str);
                    state.show_file_picker = false;
                }
            }
        }
        _ => {}
    }
}

fn apply_picker_selection(state: &mut AppState, selected_path: &str) {
    match state.file_picker.target_input {
        ActiveInput::MusicPath => state.music_path_input = selected_path.to_string(),
        ActiveInput::LyricsPath => state.lyrics_path_input = selected_path.to_string(),
        ActiveInput::OutputPath => state.output_path_input = selected_path.to_string(),
        ActiveInput::Threshold => {}
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
    let clean_path = input_path_str.trim().trim_matches('\'').trim_matches('"');
    let p = Path::new(clean_path);
    if !p.exists() {
        return None;
    }

    if p.is_dir() {
        return Some(p.to_path_buf());
    }

    if p.is_file() {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if ext == "zip" {
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
