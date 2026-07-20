# ⚡ LyricForger-Rust

```
  _                 _     ______                               
 | |               (_)   |  ____|                              
 | |    _   _ _ __  _  __| |__ ___  _ __ __ _  ___ _ __        
 | |   | | | | '__|| |/ _`  __|/ _ \| '__/ _` |/ _ \ '__|      
 | |___| |_| | |   | | (_| | | | (_) | | | (_| |  __/ |        
 |______\__, |_|   |_|\__,_|_|  \___/|_|  \__, |\___|_|        
         __/ |                             __/ |               
        |___/                             |___/                
```

> **A high-performance, modern Rust TUI utility for Android (Termux) & Linux that smartly pairs ZIP archives of music files with `.lrc`/`.txt` lyrics and embeds them directly into audio file metadata.**

[![Rust CI](https://github.com/AhooraZen/LyricForger-Rust/actions/workflows/ci.yml/badge.svg)](https://github.com/AhooraZen/LyricForger-Rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform: Termux / Android](https://img.shields.io/badge/Platform-Termux%20%7C%20Android-brightgreen.svg)]()

---

## 🇮🇷 راهنمای فارسی (Persian Documentation)

### ⚡ درباره برنامه

**LyricForger-Rust** یک ابزار ترمینالی (TUI) بسار سریع و هوشمند است که برای اندروید (Termux) و لینوکس طراحی شده است. این ابزار فایل‌های زیپ حاوی موزیک را با فایل‌های زیپ حاوی متن ترانه (`.lrc` یا `.txt`) دریافت کرده، با ۳ الگوریتم هوشمند آن‌ها را با هم جفت می‌کند و سپس متن ترانه‌ها را مستقیماً داخل متادیتای فایل‌های صوتی تزریق می‌کند تا توسط موزیک پلیرهایی مثل **Samsung Music** قابل شناسایی و نمایش باشد.

### ✨ ویژگی‌های کلیدی
- 📱 **بهینه‌شده برای Termux و اندروید:** سازگار با کیبوردهای لمسی و صفحات کوچک ترمینال.
- 🎵 **پشتیبانی کامل از Samsung Music و موزیک پلیرهای اندروید:** تزریق فریم استاندارد `USLT` در فایل‌های MP3 و تگ‌های Vorbis (`LYRICS`) در فایل‌های FLAC.
- 📦 **پشتیبانی از فایل زیپ و پوشه:** استخراج و پردازش مستقیم فایل‌های `.zip` بدون نیاز به آنزیپ دستی.
- ⚡ **سرعت بسیار بالا:** توسعه‌یافته با زبان راشت (Rust) و فریم‌ورک [Ratatui](https://github.com/ratatui/ratatui).
- 🎯 **موتور تطبیق ۳ لایه هوشمند:** پیدا کردن لیریکس مرتبط حتی در صورت تفاوت نام فایل‌ها.

### 🧠 الگوریتم‌های تطبیق هوشمند
1. **الگوریتم اول — 🎯 مطابقت دقیق پس از پاکسازی (Clean Exact Match):** حذف شماره ترک‌ها (مثل `01 - `)، حذف کلمات اضافی (مثل `[Official Audio]` یا `(Lyrics)`)، و یکسان‌سازی فاصله‌ها و حروف.
2. **الگوریتم دوم — 🏷️ مطابقت تگ متادیتا با هدر LRC:** مقایسه تگ‌های داخلی فایل صوتی (`Title` و `Artist`) با هدرهای داخل فایل لیریکس (`[ti:]` و `[ar:]`).
3. **الگوریتم سوم — 🔤 مطابقت فازی و تشابه کلمات (Fuzzy Distance):** محاسبه میزان شباهت متنی بر اساس الگوریتم‌های Levenshtein و Jaro-Winkler برای جفت‌سازی فایل‌هایی با غلط‌های املایی جزئی.

### 🚀 نحوه نصب و اجرا در ترموکس (Termux)
```bash
pkg update && pkg install -y rust git
git clone https://github.com/AhooraZen/LyricForger-Rust.git
cd LyricForger-Rust
cargo run --release
```

---

## ✨ English Features

- 📱 **Designed for Termux & Android:** Native support for small touch screens, keyboard controls, and Android MediaScanner.
- 🎵 **Samsung Music & Android Native Compatibility:** Embeds standard ID3v2 `USLT` (*Unsynchronised lyric/text information*) frames into MP3s and Vorbis `LYRICS` comments into FLAC files.
- 📦 **Zip & Folder Support:** Accepts direct `.zip` files containing music tracks and lyrics, or uncompressed local folders.
- ⚡ **Ultra Fast & Low Overhead:** Built in Rust using [Ratatui](https://github.com/ratatui/ratatui) and [Crossterm](https://github.com/crossterm-rs/crossterm).
- 🎯 **3 Multi-Layered Smart Matching Engine:** Uses 3 complementary heuristics to pair songs with lyrics even when filenames differ.

---

## 🧠 The 3 Smart Matching Strategies

1. **Strategy 1 — 🎯 Clean Normalized Exact Match (100% Score):**
   - Automatically removes leading track numbers (e.g. `01 - `, `02. `, `1 `).
   - Strips acoustic release noise keywords like `[Official Audio]`, `(Lyrics)`, `[320kbps]`, `(remix)`, `explicit`.
   - Converts underscores and hyphens to clean spaces and collapses extra whitespace.

2. **Strategy 2 — 🏷️ ID3 / Vorbis Metadata Tags vs. LRC Header Match:**
   - Reads internal audio tags (`Title`, `Artist`) directly from MP3 & FLAC frames.
   - Parses internal `.lrc` header tags (`[ti: Track Title]` and `[ar: Artist Name]`).
   - Pairs songs based on internal metadata even if filenames are completely random (`track_402.mp3` vs `song_lyrics.lrc`).

3. **Strategy 3 — 🔤 Fuzzy Distance & Token Overlap Ratio:**
   - Combines **Jaro-Winkler**, **Sorensen-Dice**, and **Token Set Ratio** string distance algorithms.
   - Matches songs with slight typos or inverted artist/title order (e.g., `Coldplay - Viva La Vida.mp3` vs `Viva La Vida.lrc`).

## 🚀 Quick Install & Auto-Updater

Run this single command in Termux or Linux to automatically clone, compile, install, and add `lyric-forger` to your PATH:

```bash
git clone https://github.com/AhooraZen/LyricForger-Rust.git && cd LyricForger-Rust && ./install.sh
```

> **Tip:** Run `./install.sh` inside the folder anytime in the future to automatically update to the latest version!

---

## 🚀 Manual Installation Options

### Option A: Install in Termux (Android)

1. Open Termux and install Rust, Cargo, and Git:
   ```bash
   pkg update && pkg install -y rust git
   ```
2. Clone this repository:
   ```bash
   git clone https://github.com/AhooraZen/LyricForger-Rust.git
   cd LyricForger-Rust
   ```
3. Build and run:
   ```bash
   cargo run --release
   ```

### Option B: Install on Linux / macOS

```bash
git clone https://github.com/AhooraZen/LyricForger-Rust.git
cd LyricForger-Rust
cargo build --release
./target/release/lyric_forger
```

### Option C: Download Pre-compiled Android Binaries (GitHub Actions)

1. Go to the [Actions Tab on GitHub](https://github.com/AhooraZen/LyricForger-Rust/actions/workflows/build-android.yml).
2. Select **Build Android Executable** from the workflows list on the left.
3. Click the **Run workflow** button on the right.
4. Once completed, download pre-compiled standalone Android binaries (`aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`) directly from the build **Artifacts** section!

---

## 🎮 TUI Keybindings & Controls

| Screen | Keybinding | Action |
| :--- | :--- | :--- |
| **Setup** | `Tab` / `Shift+Tab` | Switch active input field (Music ZIP ➔ Lyrics ZIP ➔ Threshold) |
| **Setup** | `Type Text` | Edit path or confidence threshold percentage |
| **Setup** | `Enter` | Execute Zip Extraction & Smart Matching Engine |
| **Analysis** | `↑` / `↓` Arrows | Scroll through table of matched song/lyric pairs |
| **Analysis** | `Spacebar` | Toggle filter to display unmatched songs only |
| **Analysis** | `Enter` | Start embedding lyrics into music files |
| **Global** | `H` or `?` | Open / Close Help Dialog Modal |
| **Global** | `Esc` / `Ctrl+C` | Return to previous screen or quit |

---

## 📱 How to view lyrics in Samsung Music

1. Run **LyricForger-Rust** and complete the forging process.
2. If lyrics do not show up immediately in Samsung Music, restart Samsung Music or go to **Settings ➔ Applications ➔ Samsung Music ➔ Clear Cache** to force Android's `MediaScanner` to re-index audio tags.
3. While playing a song in Samsung Music, tap on the **Album Art / Track Title** to expand the synchronized/unsynchronized lyrics view!

---

## 📄 License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for more details.
