use super::FileLister;
use crate::utils::color::*;
use crate::utils::icons::format_with_icon;
use crate::{error::Result, theme::color_value_to_color};
use colored::*;
use crossbeam_channel::{bounded, Receiver as CReceiver};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{self},
    terminal::{self, ClearType},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::WalkBuilder;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, stdout, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::{self, Receiver},
    Arc, RwLock,
};
use std::thread;
use std::time::{Duration, SystemTime};

fn truncate_to_terminal_width(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        text.to_string()
    } else {
        let mut truncated = text[..max_width.saturating_sub(3)].to_string();
        truncated.push_str("...");
        truncated
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

    fn render(&self, terminal_width: u16) -> String {
        let theme = get_theme();
        let prompt = "  🔍 ".to_string();
        let input_field = format!("{}", self.query);
        let cursor = if self.cursor_pos == self.query.len() {
            "█"
        } else {
            " "
        };
        let content_len = prompt.len() + input_field.len() + cursor.len() + 4;
        let padding = " ".repeat((terminal_width as usize).saturating_sub(content_len));

        format!(
            "{}{}{}{}",
            prompt.color(color_value_to_color(&theme.colors.permission_none)),
            input_field
                .color(color_value_to_color(&theme.colors.file))
                .bold(),
            cursor.color(color_value_to_color(&theme.colors.permission_exec)),
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
    results: Vec<(i64, PathBuf)>,
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

    fn update_results(&mut self, results: Vec<(i64, PathBuf)>) {
        self.results = results;
        self.selected_idx = self.selected_idx.min(self.results.len().saturating_sub(1));
        self.update_window();
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

    fn format_path_display(&self, path: &Path, is_selected: bool) -> (String, String) {
        let theme = get_theme();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let file_display = if is_selected {
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
            format_with_icon(
                path,
                file_name
                    .color(color_value_to_color(&theme.colors.file))
                    .to_string(),
                true,
            )
        };

        let path_display = format!(
            "  {}",
            path.display()
                .to_string()
                .color(color_value_to_color(&theme.colors.permission_none))
        );

        (file_display, path_display)
    }

    fn render(&self, terminal_width: u16, indexing: bool) -> Vec<String> {
        let theme = get_theme();
        let max_width = terminal_width as usize;

        if self.results.is_empty() {
            if indexing {
                return vec![truncate_to_terminal_width(
                    &format!(
                        "  {} Indexing files ({} found)...",
                        "📂".color(color_value_to_color(&theme.colors.directory)),
                        self.total_indexed
                    )
                    .color(color_value_to_color(&theme.colors.permission_none))
                    .to_string(),
                    max_width,
                )];
            } else {
                return vec![truncate_to_terminal_width(
                    &format!(
                        "  {} No matches found.",
                        "🔍".color(color_value_to_color(&theme.colors.permission_none))
                    )
                    .color(color_value_to_color(&theme.colors.permission_none))
                    .to_string(),
                    max_width,
                )];
            }
        }

        self.results
            .iter()
            .skip(self.window_start)
            .take(self.max_visible)
            .enumerate()
            .map(|(idx, (_, path))| {
                let is_selected = idx + self.window_start == self.selected_idx;
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

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

                let metadata = path.metadata().ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

                let prefix = if is_selected {
                    "→"
                        .color(color_value_to_color(&theme.colors.directory))
                        .bold()
                } else {
                    " ".normal()
                };

                let path_str = path.display().to_string();
                let size_str = format!("{:>3}K", size / 1024);
                let date_str = chrono::Local::now().format("%b %d %H:%M").to_string();

                let fixed_elements = 10 + size_str.len() + date_str.len();
                let max_path_width = max_width.saturating_sub(fixed_elements);
                let truncated_path = truncate_to_terminal_width(&path_str, max_path_width);

                let line = format!(
                    "  {} {}  {}  {}  {}",
                    prefix,
                    name_display,
                    truncated_path.color(if is_selected {
                        color_value_to_color(&theme.colors.directory)
                    } else {
                        color_value_to_color(&theme.colors.permission_none)
                    }),
                    if is_selected {
                        colorize_size(size).bold()
                    } else {
                        colorize_size(size)
                    },
                    if is_selected {
                        colorize_date(&std::time::SystemTime::now()).bold()
                    } else {
                        colorize_date(&std::time::SystemTime::now())
                    }
                );

                truncate_to_terminal_width(&line, max_width)
            })
            .collect()
    }
}

struct StatusBar {
    indexing_complete: bool,
    total_results: usize,
    visible_range: (usize, usize),
}

impl StatusBar {
    fn new() -> Self {
        Self {
            indexing_complete: false,
            total_results: 0,
            visible_range: (0, 0),
        }
    }

    fn render(&self, terminal_width: u16, total_indexed: usize) -> String {
        let theme = get_theme();
        let status = if !self.indexing_complete {
            format!("Indexing... {}", total_indexed)
                .color(color_value_to_color(&theme.colors.executable))
        } else {
            format!("{} files indexed", total_indexed)
                .color(color_value_to_color(&theme.colors.directory))
        };

        let results_info = if self.total_results > 0 {
            if self.visible_range.1 > 0 {
                format!(
                    "{}-{} of {} matches",
                    self.visible_range.0 + 1,
                    self.visible_range.1,
                    self.total_results
                )
                .color(color_value_to_color(&theme.colors.permission_none))
            } else {
                format!("{} matches", self.total_results)
                    .color(color_value_to_color(&theme.colors.permission_none))
            }
        } else {
            "No matches".color(color_value_to_color(&theme.colors.permission_none))
        };

        let padding = " ".repeat(
            terminal_width as usize - status.to_string().len() - results_info.to_string().len() - 4,
        );

        format!("  {}{}  {}", status, padding, results_info)
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct FileEntry {
    path: PathBuf,
    path_str: String,
    name_str: String,
    modified: SystemTime,
}

impl FileEntry {
    fn new(path: PathBuf) -> Self {
        Self {
            path_str: path.to_string_lossy().into_owned(),
            name_str: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            modified: path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| SystemTime::now()),
            path,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct DirectoryIndex {
    entries: Vec<FileEntry>,
    last_updated: SystemTime,
    last_indexed_path: Option<PathBuf>,
    total_files: usize,
}

impl DirectoryIndex {
    fn new(entries: Vec<FileEntry>, last_indexed_path: Option<PathBuf>) -> Self {
        let total_files = entries.len();
        Self {
            entries,
            last_updated: SystemTime::now(),
            last_indexed_path,
            total_files,
        }
    }

    fn needs_update(&self, path: &Path) -> bool {
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified) = metadata.modified() {
                return modified > self.last_updated;
            }
        }
        true
    }

    fn validate_entries(&mut self) {
        self.entries.retain(|entry| {
            entry.path.exists()
                && entry.path.metadata().map_or(false, |m| {
                    m.modified()
                        .map_or(false, |modified| modified <= entry.modified)
                })
        });
        self.total_files = self.entries.len();
    }
}

#[derive(Clone)]
struct SearchIndex {
    entries: Arc<RwLock<Vec<FileEntry>>>,
    total_files: Arc<AtomicUsize>,
    indexing_complete: Arc<AtomicBool>,
    index_path: PathBuf,
}

impl SearchIndex {
    fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lla")
            .join("indexes");
        fs::create_dir_all(&config_dir).unwrap_or_default();

        Self {
            entries: Arc::new(RwLock::new(Vec::with_capacity(10000))),
            total_files: Arc::new(AtomicUsize::new(0)),
            indexing_complete: Arc::new(AtomicBool::new(false)),
            index_path: config_dir,
        }
    }

    fn get_index_path(&self, directory: &Path) -> PathBuf {
        let dir_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            directory.to_string_lossy().hash(&mut hasher);
            hasher.finish()
        };
        self.index_path.join(format!("{:x}.index", dir_hash))
    }

    fn load_index(&self, directory: &Path) -> Option<DirectoryIndex> {
        let index_path = self.get_index_path(directory);
        if index_path.exists() {
            if let Ok(content) = fs::read_to_string(&index_path) {
                if let Ok(index) = serde_json::from_str::<DirectoryIndex>(&content) {
                    if !index.needs_update(directory) {
                        return Some(index);
                    }
                }
            }
        }
        None
    }

    fn save_index(
        &self,
        directory: &Path,
        entries: &[FileEntry],
        last_indexed_path: Option<PathBuf>,
    ) {
        let index = DirectoryIndex::new(entries.to_vec(), last_indexed_path);
        let index_path = self.get_index_path(directory);
        if let Ok(content) = serde_json::to_string(&index) {
            fs::write(index_path, content).unwrap_or_default();
        }
    }

    fn add_entries(
        &self,
        directory: &Path,
        new_entries: Vec<FileEntry>,
        last_path: Option<PathBuf>,
    ) {
        if let Ok(mut entries) = self.entries.write() {
            entries.extend(new_entries);
            self.total_files.store(entries.len(), Ordering::SeqCst);
            self.save_index(directory, &entries, last_path);
        }
    }

    fn search(&self, query: &str, max_results: usize) -> Vec<(i64, PathBuf)> {
        let entries_guard = match self.entries.read() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };

        if query.is_empty() {
            let mut recent_entries: Vec<_> = entries_guard
                .iter()
                .map(|entry| (entry.modified, entry.path.clone()))
                .collect();
            recent_entries.sort_by(|(a, _), (b, _)| b.cmp(a));
            return recent_entries
                .into_iter()
                .take(max_results)
                .map(|(_, path)| (0, path))
                .collect();
        }

        let query_lower = query.to_lowercase();
        let query_chars: Vec<char> = query_lower.chars().collect();
        let query_first = query_chars.first().copied();

        let filtered: Vec<_> = entries_guard
            .par_iter()
            .filter(|entry| {
                if let Some(first) = query_first {
                    let entry_lower = entry.path_str.to_lowercase();
                    entry_lower.contains(first) && entry_lower.contains(&query_lower)
                } else {
                    true
                }
            })
            .collect();

        let chunk_size = (filtered.len() / rayon::current_num_threads())
            .max(100)
            .min(1000);

        let mut scored: Vec<_> = filtered
            .par_chunks(chunk_size)
            .flat_map(|chunk| {
                let matcher = SkimMatcherV2::default().ignore_case();
                chunk
                    .iter()
                    .filter_map(|entry| {
                        let name_lower = entry.name_str.to_lowercase();
                        if name_lower.contains(&query_lower) {
                            Some((i64::MAX, entry.path.clone()))
                        } else if let Some(score) = matcher.fuzzy_match(&entry.path_str, query) {
                            Some((score, entry.path.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        scored.par_sort_unstable_by(|(a, _), (b, _)| b.cmp(a));
        scored.truncate(max_results);
        scored
    }

    fn start_indexing(&self, directory: PathBuf) -> (CReceiver<usize>, Receiver<PathBuf>) {
        let entries = Arc::clone(&self.entries);
        let total_files = Arc::clone(&self.total_files);
        let indexing_complete = Arc::clone(&self.indexing_complete);
        let (progress_tx, progress_rx) = bounded(1000);
        let (path_tx, path_rx) = mpsc::channel();

        indexing_complete.store(false, Ordering::SeqCst);

        if let Some(mut index) = self.load_index(&directory) {
            index.validate_entries();

            if let Ok(mut entries_guard) = entries.write() {
                entries_guard.clear();
                entries_guard.extend(index.entries);
                total_files.store(entries_guard.len(), Ordering::SeqCst);
                let _ = progress_tx.send(entries_guard.len());

                let start_path = index.last_indexed_path;
                indexing_complete.store(false, Ordering::SeqCst);
                self.resume_index_in_background(
                    directory.clone(),
                    entries.clone(),
                    total_files.clone(),
                    path_tx.clone(),
                    start_path,
                );
                return (progress_rx, path_rx);
            }
        }

        if let Ok(mut entries_guard) = entries.write() {
            entries_guard.clear();
        }

        let index = Arc::new(self.clone());
        let directory_clone = directory.clone();

        thread::spawn(move || {
            let (sender, receiver) = bounded(10000);
            let progress_tx = progress_tx.clone();

            thread::spawn(move || {
                let walker = WalkBuilder::new(&directory)
                    .hidden(false)
                    .git_ignore(false)
                    .ignore(false)
                    .parents(false)
                    .max_depth(None)
                    .follow_links(true)
                    .same_file_system(false)
                    .build_parallel();

                walker.run(|| {
                    let sender = sender.clone();
                    Box::new(move |entry| {
                        if let Ok(entry) = entry {
                            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                                if sender.send(entry.into_path()).is_err() {
                                    return ignore::WalkState::Quit;
                                }
                            }
                        }
                        ignore::WalkState::Continue
                    })
                });
            });

            let mut batch = Vec::with_capacity(1000);
            let mut current_path = None;

            while let Ok(path) = receiver.recv_timeout(Duration::from_secs(1)) {
                batch.push(FileEntry::new(path.clone()));
                current_path = Some(path.clone());
                let _ = path_tx.send(path);

                if batch.len() >= 1000 {
                    index.add_entries(&directory_clone, batch, current_path);
                    let _ = progress_tx.send(total_files.load(Ordering::SeqCst));
                    batch = Vec::with_capacity(1000);
                    current_path = None;
                }
            }

            if !batch.is_empty() {
                index.add_entries(&directory_clone, batch, current_path);
                let _ = progress_tx.send(total_files.load(Ordering::SeqCst));
            }

            indexing_complete.store(true, Ordering::SeqCst);
        });

        (progress_rx, path_rx)
    }

    fn resume_index_in_background(
        &self,
        directory: PathBuf,
        entries: Arc<RwLock<Vec<FileEntry>>>,
        total_files: Arc<AtomicUsize>,
        path_tx: mpsc::Sender<PathBuf>,
        start_path: Option<PathBuf>,
    ) {
        thread::spawn(move || {
            let mut walker = WalkBuilder::new(&directory)
                .hidden(false)
                .git_ignore(false)
                .ignore(false)
                .build_parallel();

            if let Some(ref start) = start_path {
                if let Some(parent) = start.parent() {
                    walker = WalkBuilder::new(parent)
                        .hidden(false)
                        .git_ignore(false)
                        .ignore(false)
                        .build_parallel();
                }
            }

            let (sender, receiver) = bounded(10000);
            let mut started = start_path.is_none();

            walker.run(|| {
                let sender = sender.clone();
                let start_path = start_path.clone();
                Box::new(move |entry| {
                    if let Ok(entry) = entry {
                        if !started {
                            if let Some(ref start) = start_path {
                                if entry.path() >= start {
                                    started = true;
                                } else {
                                    return ignore::WalkState::Continue;
                                }
                            }
                        }
                        if entry.file_type().map_or(false, |ft| ft.is_file()) {
                            if sender.send(entry.into_path()).is_err() {
                                return ignore::WalkState::Quit;
                            }
                        }
                    }
                    ignore::WalkState::Continue
                })
            });

            let mut new_entries = Vec::new();

            while let Ok(path) = receiver.recv_timeout(Duration::from_secs(1)) {
                new_entries.push(FileEntry::new(path.clone()));
                let _ = path_tx.send(path);

                if new_entries.len() >= 1000 {
                    if let Ok(mut entries_guard) = entries.write() {
                        entries_guard.extend(new_entries);
                        total_files.store(entries_guard.len(), Ordering::SeqCst);
                    }
                    new_entries = Vec::new();
                }
            }

            if !new_entries.is_empty() {
                if let Ok(mut entries_guard) = entries.write() {
                    entries_guard.extend(new_entries);
                    total_files.store(entries_guard.len(), Ordering::SeqCst);
                }
            }
        });
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
        let mut status_bar = StatusBar::new();
        let mut last_render = std::time::Instant::now();

        let (_progress_rx, path_rx) = self.index.start_indexing(PathBuf::from(directory));

        let initial_results = self.index.search("", 1000);
        result_list.update_results(initial_results);
        status_bar.total_results = result_list.results.len();
        status_bar.indexing_complete = !self.index.indexing_complete.load(Ordering::SeqCst);
        status_bar.visible_range = (0, result_list.max_visible.min(result_list.results.len()));

        let mut selected_paths = Vec::new();
        let mut last_update = std::time::Instant::now();
        let mut last_index_update = std::time::Instant::now();
        let search_update_interval = Duration::from_millis(100);
        let index_update_interval = Duration::from_millis(500);
        let render_interval = Duration::from_millis(33);
        let mut last_query = String::new();
        let max_results = 1000;

        self.render_ui(&search_bar, &result_list, &status_bar)?;

        loop {
            let now = std::time::Instant::now();
            let mut should_render = false;

            if now.duration_since(last_index_update) >= index_update_interval {
                let mut new_paths = 0;
                while let Ok(_) = path_rx.try_recv() {
                    new_paths += 1;
                    if new_paths >= 1000 {
                        break;
                    }
                }
                if new_paths > 0 {
                    result_list.total_indexed = self.index.total_files.load(Ordering::SeqCst);
                    if search_bar.query.is_empty() {
                        let results = self.index.search("", max_results);
                        result_list.update_results(results);
                        status_bar.total_results = result_list.results.len();
                    }
                    last_index_update = now;
                    should_render = true;
                }
            }

            let query = &search_bar.query;
            if !query.is_empty()
                && (query != &last_query
                    || now.duration_since(last_update) >= search_update_interval)
            {
                let results = self.index.search(query, max_results);
                result_list.update_results(results);
                last_query = query.clone();
                last_update = now;
                should_render = true;

                status_bar.indexing_complete = !self.index.indexing_complete.load(Ordering::SeqCst);
                status_bar.total_results = result_list.results.len();
                status_bar.visible_range = (
                    result_list.window_start,
                    (result_list.window_start + result_list.max_visible)
                        .min(result_list.results.len()),
                );
            }

            if should_render && now.duration_since(last_render) >= render_interval {
                self.render_ui(&search_bar, &result_list, &status_bar)?;
                last_render = now;
            }

            if event::poll(Duration::from_millis(1))? {
                if let Event::Key(key) = event::read()? {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL)
                        | (KeyCode::Esc, KeyModifiers::NONE) => {
                            break;
                        }
                        (KeyCode::Up, KeyModifiers::NONE) => {
                            result_list.move_selection(-1);
                            self.render_ui(&search_bar, &result_list, &status_bar)?;
                        }
                        (KeyCode::Down, KeyModifiers::NONE) => {
                            result_list.move_selection(1);
                            self.render_ui(&search_bar, &result_list, &status_bar)?;
                        }
                        (KeyCode::Enter, KeyModifiers::NONE) => {
                            if let Some((_, path)) =
                                result_list.results.get(result_list.selected_idx)
                            {
                                execute!(
                                    stdout,
                                    terminal::Clear(ClearType::All),
                                    cursor::MoveTo(0, 0)
                                )?;
                                let (file_display, path_display) =
                                    result_list.format_path_display(path, true);
                                println!("\n  Selected: {}{}\n", file_display, path_display);
                                selected_paths.push(path.clone());
                                break;
                            }
                        }
                        (KeyCode::PageUp, KeyModifiers::NONE) => {
                            result_list.move_selection(-(result_list.max_visible as i32));
                            self.render_ui(&search_bar, &result_list, &status_bar)?;
                        }
                        (KeyCode::PageDown, KeyModifiers::NONE) => {
                            result_list.move_selection(result_list.max_visible as i32);
                            self.render_ui(&search_bar, &result_list, &status_bar)?;
                        }
                        (KeyCode::Home, KeyModifiers::NONE) => {
                            result_list.selected_idx = 0;
                            result_list.update_window();
                            self.render_ui(&search_bar, &result_list, &status_bar)?;
                        }
                        (KeyCode::End, KeyModifiers::NONE) => {
                            result_list.selected_idx = result_list.results.len().saturating_sub(1);
                            result_list.update_window();
                            self.render_ui(&search_bar, &result_list, &status_bar)?;
                        }
                        _ => {
                            if search_bar.handle_input(key.code, key.modifiers) {
                                result_list.selected_idx = 0;
                                result_list.window_start = 0;
                                last_update = now - search_update_interval;
                                self.render_ui(&search_bar, &result_list, &status_bar)?;
                            }
                        }
                    }
                }
            }
        }

        execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;

        Ok(selected_paths)
    }

    fn render_ui(
        &self,
        search_bar: &SearchBar,
        result_list: &ResultList,
        status_bar: &StatusBar,
    ) -> io::Result<()> {
        let mut stdout = stdout();
        let (width, height) = terminal::size()?;

        let search_bar_content = search_bar.render(width);
        let separator = "─".repeat(width as usize).bright_black();
        let available_height = height.saturating_sub(4) as usize;
        let result_lines =
            result_list.render(width, !self.index.indexing_complete.load(Ordering::SeqCst));
        let status_bar_content =
            status_bar.render(width, self.index.total_files.load(Ordering::SeqCst));

        execute!(
            stdout,
            cursor::SavePosition,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            style::Print(format!("{}\n{}\n", search_bar_content, separator))
        )?;

        for (i, line) in result_lines.iter().take(available_height).enumerate() {
            execute!(
                stdout,
                cursor::MoveTo(0, (i + 2) as u16),
                terminal::Clear(ClearType::CurrentLine),
                style::Print(line),
                style::Print("\n")
            )?;
        }

        execute!(
            stdout,
            cursor::MoveTo(0, height - 1),
            terminal::Clear(ClearType::CurrentLine),
            style::Print(status_bar_content)
        )?;

        execute!(
            stdout,
            cursor::MoveTo((search_bar.cursor_pos + 4) as u16, 0)
        )?;

        stdout.flush()?;
        Ok(())
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
