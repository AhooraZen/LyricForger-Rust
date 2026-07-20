use regex::Regex;
use strsim::{jaro_winkler, sorensen_dice};
use std::collections::HashSet;
use crate::types::{LyricFile, MatchResult, MatchStrategy, MusicFile};

pub struct MatcherEngine;

impl MatcherEngine {
    /// Clean and normalize filenames for smart comparison
    pub fn clean_string(input: &str) -> String {
        let mut text = input.to_lowercase();

        // Strip file extensions if present
        for ext in &[".mp3", ".flac", ".m4a", ".ogg", ".lrc", ".txt", ".wav", ".aac"] {
            if text.ends_with(ext) {
                text = text[..text.len() - ext.len()].to_string();
            }
        }

        // Regex for stripping leading track numbers (e.g. "01 - ", "01. ", "1 - ")
        if let Ok(re_track) = Regex::new(r"^\d+[\s._\-]+") {
            text = re_track.replace(&text, "").to_string();
        }

        // Regex for removing common release noise keywords like [Official Audio], (Lyrics), etc.
        if let Ok(re_tags) = Regex::new(r"(?i)[\(\[\{].*?(official|lyrics?|audio|video|remix|hd|320kbps|explicit|feat\.|ft\.).*?[\)\]\}]") {
            text = re_tags.replace_all(&text, "").to_string();
        }

        // Replace punctuation, underscores, and hyphens with spaces
        let cleaned: String = text
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect();

        // Normalize collapse multiple spaces
        cleaned.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    /// Helper to get unique word tokens
    fn get_tokens(s: &str) -> HashSet<String> {
        s.split_whitespace().map(|w| w.to_string()).collect()
    }

    /// Strategy 1: Clean Normalized Exact Match (Score 100)
    pub fn strategy_exact_clean(music: &MusicFile, lyric: &LyricFile) -> Option<u32> {
        let m_clean = Self::clean_string(&music.filename);
        let l_clean = Self::clean_string(&lyric.filename);

        if !m_clean.is_empty() && m_clean == l_clean {
            return Some(100);
        }

        if music.clean_name == lyric.clean_name && !music.clean_name.is_empty() {
            return Some(100);
        }

        None
    }

    /// Strategy 2: ID3 / Vorbis Metadata Tags vs LRC Header Tag Match
    pub fn strategy_metadata_header(music: &MusicFile, lyric: &LyricFile) -> Option<u32> {
        let mut score = 0u32;
        let mut checks = 0u32;

        if let (Some(m_title), Some(l_title)) = (&music.title, &lyric.title_header) {
            let mt = Self::clean_string(m_title);
            let lt = Self::clean_string(l_title);
            checks += 1;
            if mt == lt && !mt.is_empty() {
                score += 50;
            } else if jaro_winkler(&mt, &lt) > 0.85 {
                score += 40;
            }
        }

        if let (Some(m_artist), Some(l_artist)) = (&music.artist, &lyric.artist_header) {
            let ma = Self::clean_string(m_artist);
            let la = Self::clean_string(l_artist);
            checks += 1;
            if ma == la && !ma.is_empty() {
                score += 50;
            } else if jaro_winkler(&ma, &la) > 0.85 {
                score += 40;
            }
        }

        if checks > 0 && score >= 40 {
            Some(score.min(100))
        } else {
            None
        }
    }

    /// Strategy 3: Fuzzy Distance & Token Overlap (Jaro-Winkler / Sorensen-Dice / Token Set)
    pub fn strategy_fuzzy_similarity(music: &MusicFile, lyric: &LyricFile) -> u32 {
        let m_clean = Self::clean_string(&music.filename);
        let l_clean = Self::clean_string(&lyric.filename);

        if m_clean.is_empty() || l_clean.is_empty() {
            return 0;
        }

        let jw_sim = jaro_winkler(&m_clean, &l_clean);
        let sd_sim = sorensen_dice(&m_clean, &l_clean);

        let m_tokens = Self::get_tokens(&m_clean);
        let l_tokens = Self::get_tokens(&l_clean);
        let intersection = m_tokens.intersection(&l_tokens).count();
        let union = m_tokens.union(&l_tokens).count();
        let token_ratio = if union > 0 {
            intersection as f64 / union as f64
        } else {
            0.0
        };

        let final_score = (jw_sim * 0.40 + sd_sim * 0.35 + token_ratio * 0.25) * 100.0;
        final_score.round() as u32
    }

    /// Run all 3 strategies and find the best matching LRC/TXT for each Music file
    pub fn find_best_matches(
        music_files: &[MusicFile],
        lyric_files: &[LyricFile],
        threshold: u32,
    ) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for music in music_files {
            let mut best_lyric_id: Option<usize> = None;
            let mut best_score: u32 = 0;
            let mut best_strategy: Option<MatchStrategy> = None;
            let mut best_detail = String::from("No match found");

            for lyric in lyric_files {
                // Check Strategy 1: Exact Clean
                if let Some(score) = Self::strategy_exact_clean(music, lyric) {
                    if score > best_score {
                        best_score = score;
                        best_lyric_id = Some(lyric.id);
                        best_strategy = Some(MatchStrategy::ExactClean);
                        best_detail = format!("Matched via exact clean filename: '{}'", lyric.filename);
                    }
                }

                // Check Strategy 2: Tag / Header Metadata
                if let Some(score) = Self::strategy_metadata_header(music, lyric) {
                    if score > best_score {
                        best_score = score;
                        best_lyric_id = Some(lyric.id);
                        best_strategy = Some(MatchStrategy::MetadataHeader);
                        best_detail = format!("Matched via ID3/LRC metadata tags: '{}'", lyric.filename);
                    }
                }

                // Check Strategy 3: Fuzzy Distance
                let fuzzy_score = Self::strategy_fuzzy_similarity(music, lyric);
                if fuzzy_score >= threshold && fuzzy_score > best_score {
                    best_score = fuzzy_score;
                    best_lyric_id = Some(lyric.id);
                    best_strategy = Some(MatchStrategy::FuzzySimilarity);
                    best_detail = format!("Matched via fuzzy similarity ({}%): '{}'", fuzzy_score, lyric.filename);
                }
            }

            if best_score < threshold {
                best_lyric_id = None;
                best_strategy = None;
                best_detail = format!("Below threshold (best confidence was {}%)", best_score);
            }

            results.push(MatchResult {
                music_id: music.id,
                lyric_id: best_lyric_id,
                confidence: best_score,
                strategy: best_strategy,
                detail: best_detail,
            });
        }

        results
    }
}
