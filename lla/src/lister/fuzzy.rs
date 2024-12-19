use super::FileLister;
use crate::utils::color::*;
use crate::utils::icons::format_with_icon;
use crate::{error::Result, theme::color_value_to_color};
use colored::*;
use crossbeam_channel::bounded;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{self},
    terminal::{self, ClearType},
};
use ignore::WalkBuilder;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::Permissions;
use std::io::{self, stdout, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering},
    Arc,
};
use std::thread;
use std::time::{Duration, SystemTime};
use unicode_normalization::UnicodeNormalization;

const WORKER_THREADS: usize = 8;
const CHUNK_SIZE: usize = 1000;
const SCORE_MATCH: i32 = 16;
const SCORE_GAP_START: i32 = -3;
const SCORE_GAP_EXTENSION: i32 = -1;
const BONUS_BOUNDARY: i32 = SCORE_MATCH / 2;
const BONUS_NON_WORD: i32 = SCORE_MATCH / 2;
const BONUS_CAMEL: i32 = BONUS_BOUNDARY + SCORE_GAP_EXTENSION;
const BONUS_CONSECUTIVE: i32 = -(SCORE_GAP_START + SCORE_GAP_EXTENSION);
const BONUS_FIRST_CHAR_MULTIPLIER: i32 = 2;
const BONUS_BOUNDARY_WHITE: i32 = BONUS_BOUNDARY + 2;
const BONUS_BOUNDARY_DELIMITER: i32 = BONUS_BOUNDARY + 1;

#[allow(dead_code)]
#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
    path_str: String,
    name_str: String,
    modified: SystemTime,
    normalized_path: String,
    score_cache: Arc<RwLock<HashMap<String, (i32, Vec<usize>)>>>,
}

impl FileEntry {
    fn new(path: PathBuf) -> Self {
        let path_str = path.to_string_lossy().into_owned();
        let name_str = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let normalized_path = path_str.nfkd().collect::<String>().to_lowercase();

        Self {
            path_str,
            name_str,
            normalized_path,
            modified: path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| SystemTime::now()),
            path,
            score_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Clone)]
struct MatchResult {
    score: i32,
    positions: Vec<usize>,
    entry: FileEntry,
}

#[derive(Clone)]
struct FuzzyMatcher {
    case_sensitive: bool,
    pattern_cache: Arc<RwLock<HashMap<String, Vec<char>>>>,
    bonus_cache: Arc<RwLock<HashMap<(char, char), i32>>>,
}

impl FuzzyMatcher {
    fn new(case_sensitive: bool) -> Self {
        Self {
            case_sensitive,
            pattern_cache: Arc::new(RwLock::new(HashMap::new())),
            bonus_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get_cached_pattern(&self, pattern: &str) -> Vec<char> {
        if let Some(cached) = self.pattern_cache.read().get(pattern) {
            return cached.clone();
        }

        let normalized = if !self.case_sensitive {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        let chars: Vec<char> = normalized.chars().collect();
        self.pattern_cache
            .write()
            .insert(pattern.to_string(), chars.clone());
        chars
    }

    fn compute_bonus(&self, prev_class: char, curr_class: char) -> i32 {
        if let Some(&bonus) = self.bonus_cache.read().get(&(prev_class, curr_class)) {
            return bonus;
        }

        let bonus = match (prev_class, curr_class) {
            (' ', c) if c.is_alphanumeric() => BONUS_BOUNDARY_WHITE,
            ('/', c) | ('\\', c) | ('_', c) | ('-', c) | ('.', c) => {
                if c.is_alphanumeric() {
                    BONUS_BOUNDARY_DELIMITER
                } else {
                    BONUS_NON_WORD
                }
            }
            (p, c) if !p.is_alphanumeric() && c.is_alphanumeric() => BONUS_BOUNDARY,
            (p, c) if p.is_lowercase() && c.is_uppercase() => BONUS_CAMEL,
            (p, c) if !p.is_numeric() && c.is_numeric() => BONUS_CAMEL,
            (_, c) if !c.is_alphanumeric() => BONUS_NON_WORD,
            _ => 0,
        };

        self.bonus_cache
            .write()
            .insert((prev_class, curr_class), bonus);
        bonus
    }

    fn fuzzy_match(&self, text: &str, pattern: &str) -> Option<(i32, Vec<usize>)> {
        if pattern.is_empty() {
            return Some((0, vec![]));
        }

        let text = if !self.case_sensitive {
            text.to_lowercase()
        } else {
            text.to_string()
        };

        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars = self.get_cached_pattern(pattern);

        let m = pattern_chars.len();
        let n = text_chars.len();

        if m > n {
            return None;
        }

        let first_char = pattern_chars[0];
        if !text_chars.contains(&first_char) {
            return None;
        }

        let mut dp = vec![vec![0; n]; m];
        let mut pos = vec![vec![0; n]; m];
        let mut matches = vec![false; n];
        let mut consecutive = vec![0; n];

        let mut found_first = false;
        for (j, &tc) in text_chars.iter().enumerate() {
            if tc == first_char {
                let bonus = if j == 0 {
                    BONUS_BOUNDARY_WHITE
                } else {
                    self.compute_bonus(text_chars[j - 1], tc)
                };
                dp[0][j] = SCORE_MATCH + bonus * BONUS_FIRST_CHAR_MULTIPLIER;
                matches[j] = true;
                consecutive[j] = 1;
                found_first = true;
            } else if found_first {
                dp[0][j] = dp[0][j - 1] + SCORE_GAP_EXTENSION;
            }
        }

        if !found_first {
            return None;
        }

        for i in 1..m {
            let mut prev_score = 0;
            let mut prev_j = 0;
            let curr_char = pattern_chars[i];

            for j in i..n {
                #[allow(unused_assignments)]
                let mut score = 0;
                if text_chars[j] == curr_char {
                    let bonus = if j == 0 {
                        BONUS_BOUNDARY_WHITE
                    } else {
                        self.compute_bonus(text_chars[j - 1], text_chars[j])
                    };

                    let consec = if j > 0 && matches[j - 1] {
                        consecutive[j - 1] + 1
                    } else {
                        1
                    };
                    consecutive[j] = consec;

                    score = dp[i - 1][j - 1] + SCORE_MATCH;
                    if consec > 1 {
                        score += BONUS_CONSECUTIVE * (consec - 1) as i32;
                    }
                    score += bonus;

                    matches[j] = true;
                    prev_j = j;
                } else {
                    score = prev_score + SCORE_GAP_EXTENSION;
                    consecutive[j] = 0;
                }

                dp[i][j] = score;
                pos[i][j] = if matches[j] { j } else { prev_j };
                prev_score = score;
            }
        }

        let mut positions = Vec::with_capacity(m);
        let mut j = n - 1;
        for _i in (0..m).rev() {
            while j > 0 && !matches[j] {
                j -= 1;
            }
            if matches[j] {
                positions.push(j);
                j -= 1;
            }
        }
        positions.reverse();

        Some((dp[m - 1][n - 1], positions))
    }
}

#[derive(Clone)]
struct SearchIndex {
    entries: Arc<RwLock<Vec<FileEntry>>>,
    matcher: FuzzyMatcher,
    last_query: Arc<RwLock<String>>,
    last_results: Arc<RwLock<Vec<MatchResult>>>,
}

impl SearchIndex {
    fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            matcher: FuzzyMatcher::new(false),
            last_query: Arc::new(RwLock::new(String::new())),
            last_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn add_entries(&self, new_entries: Vec<FileEntry>) {
        let mut entries = self.entries.write();
        entries.extend(new_entries);
    }

    fn search(&self, query: &str, max_results: usize) -> Vec<MatchResult> {
        if query.is_empty() {
            let entries = self.entries.read();
            let mut results: Vec<_> = entries
                .iter()
                .map(|entry| MatchResult {
                    score: 0,
                    positions: vec![],
                    entry: entry.clone(),
                })
                .collect();

            results.par_sort_unstable_by(|a, b| {
                a.entry
                    .name_str
                    .len()
                    .cmp(&b.entry.name_str.len())
                    .then_with(|| a.entry.name_str.cmp(&b.entry.name_str))
            });
            results.truncate(max_results);
            return results;
        }

        {
            let last_query = self.last_query.read();
            if query.starts_with(&*last_query) {
                let cached_results = self.last_results.read();
                if !cached_results.is_empty() {
                    let filtered: Vec<_> = cached_results
                        .iter()
                        .filter_map(|result| {
                            self.matcher
                                .fuzzy_match(&result.entry.normalized_path, query)
                                .map(|(score, positions)| MatchResult {
                                    score,
                                    positions,
                                    entry: result.entry.clone(),
                                })
                        })
                        .collect();

                    if !filtered.is_empty() {
                        let mut results = filtered;
                        results.par_sort_unstable_by(|a, b| {
                            b.score
                                .cmp(&a.score)
                                .then_with(|| a.entry.path_str.len().cmp(&b.entry.path_str.len()))
                        });
                        results.truncate(max_results);
                        return results;
                    }
                }
            }
        }

        let entries = self.entries.read();
        let chunk_size = (entries.len() / WORKER_THREADS).max(CHUNK_SIZE);

        let results: Vec<_> = entries
            .par_chunks(chunk_size)
            .flat_map(|chunk| {
                chunk
                    .iter()
                    .filter_map(|entry| {
                        if let Some((score, positions)) = entry.score_cache.read().get(query) {
                            return Some(MatchResult {
                                score: *score,
                                positions: positions.clone(),
                                entry: entry.clone(),
                            });
                        }

                        if let Some((score, positions)) =
                            self.matcher.fuzzy_match(&entry.normalized_path, query)
                        {
                            entry
                                .score_cache
                                .write()
                                .insert(query.to_string(), (score, positions.clone()));
                            Some(MatchResult {
                                score,
                                positions,
                                entry: entry.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut results = results;
        results.par_sort_unstable_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.entry.path_str.len().cmp(&b.entry.path_str.len()))
        });
        results.truncate(max_results);

        *self.last_query.write() = query.to_string();
        *self.last_results.write() = results.clone();

        results
    }
}

pub struct FuzzyLister {
    index: SearchIndex,
}

impl FuzzyLister {
    pub fn new() -> Self {
        Self {
            index: SearchIndex::new(),
        }
    }

    fn render_ui(&self, search_bar: &SearchBar, result_list: &ResultList) -> io::Result<()> {
        let mut stdout = stdout();
        let (width, height) = terminal::size()?;
        let available_height = height.saturating_sub(4) as usize;

        static mut LAST_SEARCH_BAR: Option<String> = None;
        let search_bar_rendered = search_bar.render(width);
        let should_render_search = unsafe {
            if LAST_SEARCH_BAR.as_ref() != Some(&search_bar_rendered) {
                LAST_SEARCH_BAR = Some(search_bar_rendered.clone());
                true
            } else {
                false
            }
        };

        if should_render_search {
            execute!(
                stdout,
                cursor::MoveTo(0, 0),
                terminal::Clear(ClearType::CurrentLine),
                style::Print(&search_bar_rendered),
                cursor::MoveTo(0, 1),
                terminal::Clear(ClearType::CurrentLine),
                style::Print("─".repeat(width as usize).bright_black())
            )?;
        }

        static mut LAST_RESULTS: Option<Vec<String>> = None;
        let result_lines = result_list.render(width);

        let should_render_full = unsafe {
            if LAST_RESULTS.as_ref().map_or(true, |last| {
                last.len() != result_lines.len()
                    || last.iter().zip(result_lines.iter()).any(|(a, b)| a != b)
            }) {
                LAST_RESULTS = Some(result_lines.clone());
                true
            } else {
                false
            }
        };

        if should_render_full {
            for i in 2..height.saturating_sub(1) {
                execute!(
                    stdout,
                    cursor::MoveTo(0, i),
                    terminal::Clear(ClearType::CurrentLine)
                )?;
            }

            for (i, line) in result_lines.iter().take(available_height).enumerate() {
                execute!(
                    stdout,
                    cursor::MoveTo(0, (i + 2) as u16),
                    style::Print(line)
                )?;
            }
        }

        static mut LAST_STATUS: Option<String> = None;
        let status_line = format!(
            "{}{}{}",
            " Total: ".bold(),
            result_list.results.len().to_string().yellow(),
            format!(
                " (showing {}-{} of {})",
                result_list.window_start + 1,
                (result_list.window_start + available_height).min(result_list.results.len()),
                result_list.total_indexed
            )
            .bright_black()
        );

        let should_render_status = unsafe {
            if LAST_STATUS.as_ref() != Some(&status_line) {
                LAST_STATUS = Some(status_line.clone());
                true
            } else {
                false
            }
        };

        if should_render_status {
            execute!(
                stdout,
                cursor::MoveTo(0, height - 1),
                terminal::Clear(ClearType::CurrentLine),
                style::Print(&status_line)
            )?;
        }

        execute!(
            stdout,
            cursor::MoveTo((search_bar.cursor_pos + 4) as u16, 0)
        )?;

        stdout.flush()
    }

    fn run_interactive(
        &self,
        directory: &str,
        _recursive: bool,
        _depth: Option<usize>,
    ) -> Result<Vec<PathBuf>> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(ClearType::All)
        )?;

        let mut search_bar = SearchBar::new();
        let mut result_list = ResultList::new(terminal::size()?.1.saturating_sub(4) as usize);
        let mut selected_paths = Vec::new();

        let (sender, receiver) = bounded(50000);
        let total_indexed = Arc::new(AtomicUsize::new(0));
        let indexing_complete = Arc::new(AtomicBool::new(false));

        let index = Arc::new(self.index.clone());
        let total_indexed_clone = Arc::clone(&total_indexed);
        let indexing_complete_clone = Arc::clone(&indexing_complete);
        let directory = directory.to_string();

        thread::spawn(move || {
            let walker = WalkBuilder::new(&directory)
                .hidden(false)
                .git_ignore(false)
                .ignore(false)
                .build_parallel();

            let (tx, rx) = std::sync::mpsc::channel();

            walker.run(|| {
                let tx = tx.clone();
                Box::new(move |entry| {
                    if let Ok(entry) = entry {
                        if entry.file_type().map_or(false, |ft| ft.is_file()) {
                            let _ = tx.send(FileEntry::new(entry.into_path()));
                        }
                    }
                    ignore::WalkState::Continue
                })
            });

            let mut batch = Vec::with_capacity(1000);
            while let Ok(entry) = rx.recv() {
                batch.push(entry);
                if batch.len() >= 1000 {
                    total_indexed_clone.fetch_add(batch.len(), AtomicOrdering::SeqCst);
                    let _ = sender.send(batch);
                    batch = Vec::with_capacity(1000);
                }
            }

            if !batch.is_empty() {
                total_indexed_clone.fetch_add(batch.len(), AtomicOrdering::SeqCst);
                let _ = sender.send(batch);
            }

            indexing_complete_clone.store(true, AtomicOrdering::SeqCst);
        });

        let mut last_query = String::new();
        let mut last_update = std::time::Instant::now();
        let mut last_render = std::time::Instant::now();
        let mut last_status_update = std::time::Instant::now();
        let mut needs_render = true;
        let mut initial_load_done = false;

        let update_interval = Duration::from_millis(150);
        let render_interval = Duration::from_millis(33);
        let status_update_interval = Duration::from_millis(100);

        loop {
            let now = std::time::Instant::now();

            while let Ok(batch) = receiver.try_recv() {
                index.add_entries(batch);
                let current_indexed = total_indexed.load(AtomicOrdering::SeqCst);

                if now.duration_since(last_status_update) >= status_update_interval {
                    result_list.total_indexed = current_indexed;
                    self.render_status_bar(&result_list)?;
                    last_status_update = now;
                }

                if !initial_load_done && current_indexed > 100 {
                    let results = index.search("", 1000);
                    result_list.update_results(results);
                    initial_load_done = true;
                    needs_render = true;
                }
            }

            if event::poll(Duration::from_millis(1))? {
                if let Event::Key(key) = event::read()? {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL)
                        | (KeyCode::Esc, KeyModifiers::NONE) => break,
                        (KeyCode::Enter, KeyModifiers::NONE) => {
                            if let Some(result) = result_list.get_selected() {
                                selected_paths.push(result.entry.path.clone());
                                break;
                            }
                        }
                        (KeyCode::Up, KeyModifiers::NONE) => {
                            result_list.move_selection(-1);
                            needs_render = true;
                        }
                        (KeyCode::Down, KeyModifiers::NONE) => {
                            result_list.move_selection(1);
                            needs_render = true;
                        }
                        _ => {
                            if search_bar.handle_input(key.code, key.modifiers) {
                                last_query = search_bar.query.clone();
                                result_list.selected_idx = 0;
                                result_list.window_start = 0;

                                let results = index.search(&last_query, 1000);
                                result_list.update_results(results);
                                needs_render = true;
                                last_update = now;
                            }
                        }
                    }
                }
            }

            if !last_query.is_empty() && now.duration_since(last_update) >= update_interval {
                let results = index.search(&last_query, 1000);
                if result_list.update_results(results) {
                    needs_render = true;
                }
                last_update = now;
            }

            if needs_render && now.duration_since(last_render) >= render_interval {
                self.render_ui(&search_bar, &result_list)?;
                last_render = now;
                needs_render = false;
            }

            thread::sleep(Duration::from_millis(1));
        }

        execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;

        Ok(selected_paths)
    }

    fn render_status_bar(&self, result_list: &ResultList) -> io::Result<()> {
        let mut stdout = stdout();
        let (_, height) = terminal::size()?;
        let available_height = height.saturating_sub(4) as usize;

        let status_line = format!(
            "{}{}{}",
            " Total: ".bold(),
            result_list.results.len().to_string().yellow(),
            format!(
                " (showing {}-{} of {})",
                result_list.window_start + 1,
                (result_list.window_start + available_height).min(result_list.results.len()),
                result_list.total_indexed
            )
            .bright_black()
        );

        execute!(
            stdout,
            cursor::MoveTo(0, height - 1),
            terminal::Clear(ClearType::CurrentLine),
            style::Print(&status_line)
        )?;

        stdout.flush()
    }
}

impl FileLister for FuzzyLister {
    fn list_files(
        &self,
        directory: &str,
        recursive: bool,
        depth: Option<usize>,
    ) -> Result<Vec<PathBuf>> {
        self.run_interactive(directory, recursive, depth)
    }
}

struct SearchBar {
    query: String,
    cursor_pos: usize,
}

impl SearchBar {
    fn new() -> Self {
        Self {
            query: String::new(),
            cursor_pos: 0,
        }
    }

    fn render(&self, width: u16) -> String {
        let theme = get_theme();
        let prompt = "    ".to_string();
        let input = if self.query.is_empty() {
            "Type to search...".to_string().bright_black().to_string()
        } else {
            self.query.clone()
        };

        let cursor = if !self.query.is_empty() && self.cursor_pos == self.query.len() {
            "▎"
                .color(color_value_to_color(&theme.colors.permission_exec))
                .to_string()
        } else {
            " ".to_string()
        };

        let content_len = prompt.len() + input.len() + cursor.len() + 4;
        let padding = " ".repeat((width as usize).saturating_sub(content_len));

        let border_color = color_value_to_color(&theme.colors.permission_none);
        let input_color = if self.query.is_empty() {
            input
        } else {
            input
                .color(color_value_to_color(&theme.colors.file))
                .bold()
                .to_string()
        };

        format!(
            "{}{}{}{}",
            prompt.color(border_color),
            input_color,
            cursor,
            padding
        )
    }

    fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        match (key, modifiers) {
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.query.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                true
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.query.remove(self.cursor_pos);
                    true
                } else {
                    false
                }
            }
            (KeyCode::Left, KeyModifiers::NONE) => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    true
                } else {
                    false
                }
            }
            (KeyCode::Right, KeyModifiers::NONE) => {
                if self.cursor_pos < self.query.len() {
                    self.cursor_pos += 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

struct ResultList {
    results: Vec<MatchResult>,
    selected_idx: usize,
    window_start: usize,
    max_visible: usize,
    total_indexed: usize,
}

impl ResultList {
    fn new(max_visible: usize) -> Self {
        Self {
            results: Vec::new(),
            selected_idx: 0,
            window_start: 0,
            max_visible,
            total_indexed: 0,
        }
    }

    fn get_selected(&self) -> Option<&MatchResult> {
        self.results.get(self.selected_idx)
    }

    fn update_results(&mut self, results: Vec<MatchResult>) -> bool {
        if self.results.len() != results.len() {
            self.results = results;
            self.selected_idx = self.selected_idx.min(self.results.len().saturating_sub(1));
            self.update_window();
            return true;
        }

        let changed = self.results.iter().zip(results.iter()).any(|(a, b)| {
            a.score != b.score || a.positions != b.positions || a.entry.path != b.entry.path
        });

        if changed {
            self.results = results;
            self.selected_idx = self.selected_idx.min(self.results.len().saturating_sub(1));
            self.update_window();
        }

        changed
    }

    fn update_window(&mut self) {
        if self.selected_idx >= self.window_start + self.max_visible {
            self.window_start = self.selected_idx - self.max_visible + 1;
        } else if self.selected_idx < self.window_start {
            self.window_start = self.selected_idx;
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let new_idx = self.selected_idx as i32 + delta;
        if new_idx >= 0 && new_idx < self.results.len() as i32 {
            self.selected_idx = new_idx as usize;
            self.update_window();
        }
    }

    fn render(&self, width: u16) -> Vec<String> {
        let theme = get_theme();
        let max_width = width as usize;

        if self.results.is_empty() {
            return vec![format!(
                "  {} {}",
                "".color(color_value_to_color(&theme.colors.directory)),
                if self.total_indexed == 0 {
                    "Indexing files...".to_string()
                } else {
                    format!("No matches found (indexed {} files)", self.total_indexed)
                }
                .color(color_value_to_color(&theme.colors.permission_none))
            )];
        }

        self.results
            .iter()
            .skip(self.window_start)
            .take(self.max_visible)
            .enumerate()
            .map(|(idx, result)| {
                let is_selected = idx + self.window_start == self.selected_idx;
                let path = &result.entry.path;
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let metadata = path.metadata().ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or_else(SystemTime::now);

                let path_str = path.to_string_lossy();
                let truncated_path = if path_str.len() > max_width.saturating_sub(60) {
                    let components: Vec<_> = path.components().collect();
                    if components.len() <= 2 {
                        path_str.to_string()
                    } else {
                        let prefix = components[0..components.len() - 2]
                            .iter()
                            .map(|c| c.as_os_str().to_string_lossy())
                            .collect::<Vec<_>>();

                        if prefix.len() > 1 {
                            let first = prefix[0].to_string();
                            let last = if prefix.len() > 2 {
                                prefix.last().unwrap().to_string()
                            } else {
                                prefix[1].to_string()
                            };
                            format!("{}/.../{}", first, last)
                        } else {
                            prefix.join("/")
                        }
                        .chars()
                        .take(max_width.saturating_sub(30))
                        .collect::<String>()
                    }
                } else {
                    path_str.to_string()
                };

                let name_display = if is_selected {
                    format_with_icon(
                        path,
                        file_name
                            .color(color_value_to_color(&theme.colors.directory))
                            .bold()
                            .underline()
                            .to_string(),
                        true,
                    )
                } else {
                    format_with_icon(path, colorize_file_name(path).to_string(), true)
                };

                let prefix = if is_selected {
                    "→".bold()
                } else {
                    " ".normal()
                };

                let perms = metadata
                    .as_ref()
                    .map(|m| m.permissions())
                    .unwrap_or_else(|| Permissions::from_mode(0o644));
                let perms_display = colorize_permissions(&perms);
                let size_display = colorize_size(size);
                let date_display = colorize_date(&modified);

                format!(
                    "  {} {}  {}  {} {} {}",
                    prefix,
                    name_display,
                    truncated_path.color(if is_selected {
                        color_value_to_color(&theme.colors.directory)
                    } else {
                        color_value_to_color(&theme.colors.permission_none)
                    }),
                    perms_display,
                    size_display,
                    date_display
                )
            })
            .collect()
    }
}
