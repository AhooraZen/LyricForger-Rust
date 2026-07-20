use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchStrategy {
    ExactClean,
    MetadataHeader,
    FuzzySimilarity,
    Manual,
}

impl MatchStrategy {
    pub fn label(&self) -> &'static str {
        match self {
            MatchStrategy::ExactClean => "EXACT-CLEAN",
            MatchStrategy::MetadataHeader => "TAG-METADATA",
            MatchStrategy::FuzzySimilarity => "FUZZY-DIST",
            MatchStrategy::Manual => "MANUAL-PAIR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MusicFile {
    pub id: usize,
    pub path: PathBuf,
    pub filename: String,
    pub clean_name: String,
    pub title: Option<String>,
    pub artist: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LyricFile {
    pub id: usize,
    pub path: PathBuf,
    pub filename: String,
    pub clean_name: String,
    pub title_header: Option<String>,
    pub artist_header: Option<String>,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub music_id: usize,
    pub lyric_id: Option<usize>,
    pub confidence: u32, // 0 to 100
    pub strategy: Option<MatchStrategy>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Setup,
    Analysis,
    Forging,
    Summary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveInput {
    MusicPath,
    LyricsPath,
    OutputPath,
    Threshold,
}

#[derive(Debug, Clone)]
pub struct FilePickerEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_archive: bool,
}

pub struct FilePickerState {
    pub current_dir: PathBuf,
    pub entries: Vec<FilePickerEntry>,
    pub selected_idx: usize,
    pub target_input: ActiveInput,
}

impl Default for FilePickerState {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        Self {
            current_dir,
            entries: Vec::new(),
            selected_idx: 0,
            target_input: ActiveInput::MusicPath,
        }
    }
}

pub struct AppState {
    pub current_screen: Screen,
    pub active_input: ActiveInput,
    
    // Inputs
    pub music_path_input: String,
    pub lyrics_path_input: String,
    pub output_path_input: String,
    pub threshold: u32,
    pub error_msg: Option<String>,

    // Data
    pub music_files: Vec<MusicFile>,
    pub lyric_files: Vec<LyricFile>,
    pub matches: Vec<MatchResult>,

    // UI state
    pub selected_match_idx: usize,
    pub filter_unmatched_only: bool,
    pub show_help_modal: bool,
    pub show_file_picker: bool,
    pub file_picker: FilePickerState,

    // Execution state
    pub is_processing: bool,
    pub processed_count: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub logs: Vec<String>,

    // Temp directory handle
    pub temp_dir: Option<PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Setup,
            active_input: ActiveInput::MusicPath,
            music_path_input: String::new(),
            lyrics_path_input: String::new(),
            output_path_input: String::from("output"),
            threshold: 50,
            error_msg: None,
            music_files: Vec::new(),
            lyric_files: Vec::new(),
            matches: Vec::new(),
            selected_match_idx: 0,
            filter_unmatched_only: false,
            show_help_modal: false,
            show_file_picker: false,
            file_picker: FilePickerState::default(),
            is_processing: false,
            processed_count: 0,
            success_count: 0,
            fail_count: 0,
            logs: Vec::new(),
            temp_dir: None,
        }
    }
}
