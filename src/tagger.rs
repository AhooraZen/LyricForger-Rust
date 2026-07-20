use std::path::Path;
use id3::{frame::Lyrics, Tag, TagLike};

pub struct TaggerEngine;

impl TaggerEngine {
    /// Forge/Embed lyrics into an audio file
    pub fn embed_lyrics(audio_path: &Path, lyrics_text: &str) -> Result<(), String> {
        let ext = audio_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "mp3" => Self::embed_mp3(audio_path, lyrics_text),
            "flac" => Self::embed_flac(audio_path, lyrics_text),
            _ => {
                Self::embed_mp3(audio_path, lyrics_text)
                    .or_else(|_| Err(format!("Unsupported audio format for embedding: .{}", ext)))
            }
        }
    }

    /// Embed USLT (Unsynchronized Lyrics) ID3 frame for MP3 (Supported natively by Samsung Music)
    fn embed_mp3(audio_path: &Path, lyrics_text: &str) -> Result<(), String> {
        let mut tag = match Tag::read_from_path(audio_path) {
            Ok(t) => t,
            Err(_) => Tag::new(),
        };

        // Remove any existing USLT frames to prevent duplicates
        tag.remove("USLT");

        // Add standard USLT frame with language "eng" and empty description
        tag.add_frame(Lyrics {
            lang: String::from("eng"),
            description: String::from(""),
            text: lyrics_text.to_string(),
        });

        // Write tag back to file
        tag.write_to_path(audio_path, id3::Version::Id3v24)
            .map_err(|e| format!("Failed to write ID3 USLT frame to MP3: {}", e))?;

        Ok(())
    }

    /// Embed LYRICS & UNSYNCEDLYRICS Vorbis Comments for FLAC (Supported natively by Samsung Music & Android)
    fn embed_flac(audio_path: &Path, lyrics_text: &str) -> Result<(), String> {
        let mut tag = metaflac::Tag::read_from_path(audio_path)
            .map_err(|e| format!("Failed to read FLAC metadata: {}", e))?;

        let vorbis = tag.vorbis_comments_mut();
        
        // Clear old lyrics tags
        vorbis.remove("LYRICS");
        vorbis.remove("UNSYNCEDLYRICS");

        // Set standard Vorbis lyrics comments
        vorbis.set("LYRICS", vec![lyrics_text]);
        vorbis.set("UNSYNCEDLYRICS", vec![lyrics_text]);

        tag.save()
            .map_err(|e| format!("Failed to save FLAC vorbis comments: {}", e))?;

        Ok(())
    }

    /// Parse LRC file content for header metadata (e.g. [ti: ...], [ar: ...])
    pub fn parse_lrc_headers(content: &str) -> (Option<String>, Option<String>) {
        let mut title = None;
        let mut artist = None;

        for line in content.lines() {
            let line_trim = line.trim();
            if line_trim.starts_with("[ti:") && line_trim.ends_with(']') {
                title = Some(line_trim[4..line_trim.len() - 1].trim().to_string());
            } else if line_trim.starts_with("[ar:") && line_trim.ends_with(']') {
                artist = Some(line_trim[4..line_trim.len() - 1].trim().to_string());
            }
        }

        (title, artist)
    }

    /// Read internal ID3 or Vorbis metadata from audio file
    pub fn read_audio_tags(audio_path: &Path) -> (Option<String>, Option<String>) {
        let ext = audio_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext == "mp3" {
            if let Ok(tag) = Tag::read_from_path(audio_path) {
                let title = tag.title().map(|s| s.to_string());
                let artist = tag.artist().map(|s| s.to_string());
                return (title, artist);
            }
        } else if ext == "flac" {
            if let Ok(tag) = metaflac::Tag::read_from_path(audio_path) {
                if let Some(vorbis) = tag.vorbis_comments() {
                    let title = vorbis.title().and_then(|v| v.first()).map(|s| s.to_string());
                    let artist = vorbis.artist().and_then(|v| v.first()).map(|s| s.to_string());
                    return (title, artist);
                }
            }
        }

        (None, None)
    }
}
