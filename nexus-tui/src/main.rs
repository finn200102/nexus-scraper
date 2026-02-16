use std::collections::HashMap;
use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use nexus_core::models::Story;
use nexus_core::{resolver, sites::get_site};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_result = run_app(&mut terminal, &rt);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    app_result
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    fn frame_style(self) -> Style {
        match self {
            ThemeMode::Dark => Style::default(),
            ThemeMode::Light => Style::default().fg(Color::Black).bg(Color::White),
        }
    }

    fn accent(self) -> Style {
        match self {
            ThemeMode::Dark => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ThemeMode::Light => Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ViewMode {
    Detailed,
    Compact,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InputMode {
    Url,
    Filter,
    Jump,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Overlay {
    None,
    Help,
    Download,
    Recent,
    Bookmarks,
}

struct App {
    input_url: String,
    input_cursor: usize,
    status: String,
    story: Option<Story>,
    selected_chapter: usize,
    chapter_text: String,
    chapter_scroll: u16,
    chapter_scrolls: HashMap<usize, u16>,
    chapter_filter: String,
    filter_input: String,
    jump_input: String,
    input_mode: InputMode,
    overlay: Overlay,
    theme: ThemeMode,
    view_mode: ViewMode,
    last_error: Option<String>,
    show_error_details: bool,
    recent_urls: Vec<String>,
    bookmarks: Vec<String>,
    url_preview_scroll: u16,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input_url: String::new(),
            input_cursor: 0,
            status: "Type URL and press Enter to scrape. ? for help.".into(),
            story: None,
            selected_chapter: 0,
            chapter_text: String::new(),
            chapter_scroll: 0,
            chapter_scrolls: HashMap::new(),
            chapter_filter: String::new(),
            filter_input: String::new(),
            jump_input: String::new(),
            input_mode: InputMode::Url,
            overlay: Overlay::None,
            theme: ThemeMode::Dark,
            view_mode: ViewMode::Detailed,
            last_error: None,
            show_error_details: false,
            recent_urls: Vec::new(),
            bookmarks: Vec::new(),
            url_preview_scroll: 0,
        }
    }
}

impl App {
    fn filtered_indices(&self) -> Vec<usize> {
        let Some(story) = &self.story else { return vec![]; };
        if self.chapter_filter.trim().is_empty() {
            return (0..story.chapters.len()).collect();
        }
        let needle = self.chapter_filter.to_lowercase();
        story.chapters
            .iter()
            .enumerate()
            .filter_map(|(i, ch)| {
                let title = ch.title.as_deref().unwrap_or("untitled").to_lowercase();
                if title.contains(&needle) || format!("{}", ch.chapter_number.unwrap_or((i + 1) as u32)).contains(&needle) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn selected_story_index(&self) -> Option<usize> {
        let filtered = self.filtered_indices();
        filtered.get(self.selected_chapter).copied()
    }

    fn push_recent_url(&mut self, url: &str) {
        if url.is_empty() {
            return;
        }
        self.recent_urls.retain(|u| u != url);
        self.recent_urls.insert(0, url.to_string());
        self.recent_urls.truncate(10);
    }

    fn set_error(&mut self, msg: String) {
        self.status = format!("Error: {}", msg);
        self.last_error = Some(msg);
    }
}

fn run_app(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    rt: &tokio::runtime::Runtime,
) -> io::Result<()> {
    let mut app = App::default();

    loop {
        terminal.draw(|frame| draw_ui(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if handle_overlay_keys(&mut app, key.code) {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('?') => app.overlay = Overlay::Help,
                    KeyCode::Char('e') => app.show_error_details = !app.show_error_details,
                    KeyCode::Char('t') => {
                        app.theme = if app.theme == ThemeMode::Dark { ThemeMode::Light } else { ThemeMode::Dark }
                    }
                    KeyCode::Char('v') => {
                        app.view_mode = if app.view_mode == ViewMode::Detailed { ViewMode::Compact } else { ViewMode::Detailed }
                    }
                    KeyCode::Char('r') => app.overlay = Overlay::Recent,
                    KeyCode::Char('b') => app.overlay = Overlay::Bookmarks,
                    KeyCode::Char('m') => {
                        let u = app.input_url.trim().to_string();
                        if !u.is_empty() {
                            app.bookmarks.retain(|x| x != &u);
                            app.bookmarks.insert(0, u.clone());
                            app.bookmarks.truncate(20);
                            app.status = "Bookmarked current URL".into();
                        }
                    }
                    KeyCode::Char('/') => {
                        app.input_mode = InputMode::Filter;
                        app.filter_input = app.chapter_filter.clone();
                        app.status = "Filter mode: type and press Enter".into();
                    }
                    KeyCode::Char('g') => {
                        app.input_mode = InputMode::Jump;
                        app.jump_input.clear();
                        app.status = "Jump mode: enter chapter number then Enter".into();
                    }
                    KeyCode::Char('j') => {
                        let len = app.filtered_indices().len();
                        if len > 0 && app.selected_chapter + 1 < len {
                            persist_scroll(&mut app);
                            app.selected_chapter += 1;
                            restore_scroll(&mut app);
                        }
                    }
                    KeyCode::Char('k') => {
                        if app.selected_chapter > 0 {
                            persist_scroll(&mut app);
                            app.selected_chapter -= 1;
                            restore_scroll(&mut app);
                        }
                    }
                    KeyCode::PageDown => app.chapter_scroll = app.chapter_scroll.saturating_add(20),
                    KeyCode::PageUp => app.chapter_scroll = app.chapter_scroll.saturating_sub(20),
                    KeyCode::Down => app.chapter_scroll = app.chapter_scroll.saturating_add(2),
                    KeyCode::Up => app.chapter_scroll = app.chapter_scroll.saturating_sub(2),
                    KeyCode::Home => app.chapter_scroll = 0,
                    KeyCode::End => app.chapter_scroll = u16::MAX.saturating_sub(1),
                    KeyCode::Char('d') => app.overlay = Overlay::Download,
                    KeyCode::Char('l') => match rt.block_on(load_selected_chapter_text(&app)) {
                        Ok(text) => {
                            app.chapter_text = text;
                            app.chapter_scroll = 0;
                            app.status = "Loaded chapter text".into();
                        }
                        Err(e) => app.set_error(e),
                    },
                    KeyCode::Enter => match app.input_mode {
                        InputMode::Url => {
                            let url = app.input_url.trim().to_string();
                            if url.is_empty() {
                                app.status = "Please enter a URL".into();
                                continue;
                            }
                            app.status = "Scraping online...".into();
                            match rt.block_on(fetch_story(&url)) {
                                Ok(story) => {
                                    app.push_recent_url(&url);
                                    app.selected_chapter = 0;
                                    app.chapter_scroll = 0;
                                    app.chapter_text.clear();
                                    app.status = format!(
                                        "Loaded '{}' with {} chapters",
                                        story.story_name.clone().unwrap_or_else(|| "Unknown title".into()),
                                        story.chapters.len()
                                    );
                                    app.story = Some(story);
                                }
                                Err(e) => app.set_error(e),
                            }
                        }
                        InputMode::Filter => {
                            app.chapter_filter = app.filter_input.clone();
                            app.selected_chapter = 0;
                            app.input_mode = InputMode::Url;
                            app.status = if app.chapter_filter.is_empty() {
                                "Chapter filter cleared".into()
                            } else {
                                format!("Filter applied: {}", app.chapter_filter)
                            };
                        }
                        InputMode::Jump => {
                            let parsed = app.jump_input.parse::<usize>();
                            match parsed {
                                Ok(n) if n > 0 => {
                                    let filtered = app.filtered_indices();
                                    if n - 1 < filtered.len() {
                                        app.selected_chapter = n - 1;
                                        app.chapter_scroll = 0;
                                        app.status = format!("Jumped to chapter {}", n);
                                    } else {
                                        app.set_error(format!("Chapter {} is out of range", n));
                                    }
                                }
                                _ => app.set_error("Invalid chapter number".into()),
                            }
                            app.input_mode = InputMode::Url;
                        }
                    },
                    KeyCode::Esc => {
                        app.overlay = Overlay::None;
                        if app.input_mode != InputMode::Url {
                            app.input_mode = InputMode::Url;
                            app.status = "Cancelled input mode".into();
                        }
                    }
                    _ => handle_text_input(&mut app, key.code, key.modifiers),
                }
            }
        }
    }
}

fn handle_overlay_keys(app: &mut App, code: KeyCode) -> bool {
    match app.overlay {
        Overlay::None => false,
        Overlay::Help => {
            if matches!(code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter) {
                app.overlay = Overlay::None;
                return true;
            }
            true
        }
        Overlay::Recent => match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.overlay = Overlay::None;
                true
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let i = c.to_digit(10).unwrap_or_default() as usize;
                if i > 0 && i <= app.recent_urls.len() {
                    let url = app.recent_urls[i - 1].clone();
                    app.input_url = url;
                    app.input_cursor = app.input_url.len();
                    app.overlay = Overlay::None;
                    app.status = "Loaded URL from recent list".into();
                }
                true
            }
            _ => true,
        },
        Overlay::Bookmarks => match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.overlay = Overlay::None;
                true
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let i = c.to_digit(10).unwrap_or_default() as usize;
                if i > 0 && i <= app.bookmarks.len() {
                    let url = app.bookmarks[i - 1].clone();
                    app.input_url = url;
                    app.input_cursor = app.input_url.len();
                    app.overlay = Overlay::None;
                    app.status = "Loaded URL from bookmarks".into();
                }
                true
            }
            _ => true,
        },
        Overlay::Download => match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.overlay = Overlay::None;
                true
            }
            KeyCode::Char('1') => {
                match download_story_json(&app.story) {
                    Ok(path) => app.status = format!("Downloaded JSON to {}", path),
                    Err(e) => app.set_error(e),
                }
                app.overlay = Overlay::None;
                true
            }
            KeyCode::Char('2') => {
                match download_chapter_txt_bundle(&app.story) {
                    Ok(path) => app.status = format!("Downloaded TXT bundle to {}", path),
                    Err(e) => app.set_error(e),
                }
                app.overlay = Overlay::None;
                true
            }
            KeyCode::Char('3') => {
                match download_selected_chapter_txt(app) {
                    Ok(path) => app.status = format!("Downloaded chapter to {}", path),
                    Err(e) => app.set_error(e),
                }
                app.overlay = Overlay::None;
                true
            }
            _ => true,
        },
    }
}

fn handle_text_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match app.input_mode {
        InputMode::Url => handle_edit_with_cursor(&mut app.input_url, &mut app.input_cursor, code, modifiers, false),
        InputMode::Filter => {
            let mut cursor = app.filter_input.len();
            handle_edit_with_cursor(&mut app.filter_input, &mut cursor, code, modifiers, false);
        }
        InputMode::Jump => {
            let mut cursor = app.jump_input.len();
            handle_edit_with_cursor(&mut app.jump_input, &mut cursor, code, modifiers, true);
        }
    }

    if app.input_mode == InputMode::Url {
        let visible_width = 90usize;
        if app.input_cursor > visible_width {
            app.url_preview_scroll = (app.input_cursor - visible_width) as u16;
        } else {
            app.url_preview_scroll = 0;
        }
    }
}

fn handle_edit_with_cursor(
    target: &mut String,
    cursor: &mut usize,
    code: KeyCode,
    modifiers: KeyModifiers,
    digits_only: bool,
) {
    match code {
        KeyCode::Left => {
            if *cursor > 0 {
                *cursor -= 1;
            }
        }
        KeyCode::Right => {
            if *cursor < target.len() {
                *cursor += 1;
            }
        }
        KeyCode::Home => *cursor = 0,
        KeyCode::End => *cursor = target.len(),
        KeyCode::Backspace => {
            if *cursor > 0 {
                target.remove(*cursor - 1);
                *cursor -= 1;
            }
        }
        KeyCode::Delete => {
            if *cursor < target.len() {
                target.remove(*cursor);
            }
        }
        KeyCode::Char(c) => {
            if digits_only && !c.is_ascii_digit() {
                return;
            }
            if modifiers.contains(KeyModifiers::CONTROL) {
                return;
            }
            target.insert(*cursor, c);
            *cursor += 1;
        }
        _ => {}
    }
}

fn draw_ui(frame: &mut ratatui::Frame, app: &App) {
    let style = app.theme.frame_style();
    frame.render_widget(Block::default().style(style), frame.area());

    let constraints = if app.view_mode == ViewMode::Detailed {
        vec![
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(1),
        ]
    } else {
        vec![Constraint::Length(3), Constraint::Length(2), Constraint::Min(8), Constraint::Length(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    let input_title = match app.input_mode {
        InputMode::Url => "Story URL (type + Enter)",
        InputMode::Filter => "Filter Mode (Enter apply, Esc cancel)",
        InputMode::Jump => "Jump Mode (chapter # + Enter)",
    };

    let input_value = match app.input_mode {
        InputMode::Url => app.input_url.clone(),
        InputMode::Filter => app.filter_input.clone(),
        InputMode::Jump => app.jump_input.clone(),
    };

    let input = Paragraph::new(input_value)
        .style(style)
        .block(Block::default().title(input_title).borders(Borders::ALL));
    frame.render_widget(input, chunks[0]);

    let mut body_start = 2;

    if app.view_mode == ViewMode::Detailed {
        let url_preview = Paragraph::new(app.input_url.as_str())
            .style(style)
            .block(Block::default().title("URL Preview (full, wrapped)").borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .scroll((app.url_preview_scroll, 0));
        frame.render_widget(url_preview, chunks[1]);

        let status = Paragraph::new(app.status.as_str())
            .style(style)
            .block(Block::default().title("Status").borders(Borders::ALL));
        frame.render_widget(status, chunks[2]);

        let env_url = std::env::var("FLARESOLVERR_URL").unwrap_or_else(|_| "http://localhost:8191/v1 (default)".into());
        let meta_line = format!(
            "Filter: {} | Theme: {:?} | View: {:?} | FLARESOLVERR_URL: {}",
            if app.chapter_filter.is_empty() { "<none>" } else { &app.chapter_filter },
            app.theme,
            app.view_mode,
            env_url
        );
        let meta = Paragraph::new(meta_line)
            .style(style)
            .block(Block::default().title("Session").borders(Borders::ALL));
        frame.render_widget(meta, chunks[3]);

        body_start = 4;
    } else {
        let status = Paragraph::new(app.status.as_str())
            .style(style)
            .block(Block::default().title("Status").borders(Borders::ALL));
        frame.render_widget(status, chunks[1]);
    }

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[body_start]);

    let filtered = app.filtered_indices();
    let items: Vec<ListItem> = app
        .story
        .as_ref()
        .map(|s| {
            filtered
                .iter()
                .map(|idx| {
                    let ch = &s.chapters[*idx];
                    let title = ch.title.clone().unwrap_or_else(|| "Untitled".into());
                    let number = ch.chapter_number.unwrap_or((*idx + 1) as u32);
                    ListItem::new(format!("{:>3}. {}", number, title))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(app.selected_chapter.min(items.len().saturating_sub(1))));
    }

    let chapters = List::new(items)
        .highlight_style(app.theme.accent())
        .highlight_symbol("> ")
        .block(Block::default().title("Chapters (j/k, / filter, g jump)").borders(Borders::ALL));
    frame.render_stateful_widget(chapters, body[0], &mut state);

    let mut lines = Vec::new();
    if let Some(story) = &app.story {
        lines.push(Line::from(vec![
            Span::styled("Title: ", app.theme.accent()),
            Span::raw(story.story_name.clone().unwrap_or_else(|| "Unknown".into())),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Author: ", app.theme.accent()),
            Span::raw(story.author_name.clone().unwrap_or_else(|| "Unknown".into())),
        ]));
        lines.push(Line::from(vec![Span::styled("Site: ", app.theme.accent()), Span::raw(story.site.clone())]));
        lines.push(Line::from(vec![
            Span::styled("Chapters: ", app.theme.accent()),
            Span::raw(story.chapters.len().to_string()),
        ]));

        if !story.tags.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Tags: ", app.theme.accent()),
                Span::raw(story.tags.join(", ")),
            ]));
        }

        if let Some(desc) = &story.description {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Description:", app.theme.accent())));
            lines.extend(Text::from(desc.as_str()).lines);
        }

        lines.push(Line::from(""));
        if !app.chapter_text.is_empty() {
            lines.push(Line::from(Span::styled("Chapter text:", app.theme.accent())));
            lines.push(Line::from(""));
            lines.extend(Text::from(app.chapter_text.as_str()).lines);
        } else {
            lines.push(Line::from("Press 'l' to load selected chapter text online."));
            lines.push(Line::from("Press 'd' for download options."));
        }
    } else {
        lines.push(Line::from("No story loaded yet."));
    }

    let reader = Paragraph::new(lines)
        .style(style)
        .block(Block::default().title("Reader").borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .scroll((app.chapter_scroll, 0));
    frame.render_widget(reader, body[1]);

    if let Some(err) = &app.last_error {
        let err_text = if app.show_error_details {
            format!("{}\n(press 'e' to hide details)", err)
        } else {
            "Error present (press 'e' for details)".into()
        };
        let err_box = Paragraph::new(err_text)
            .style(Style::default().fg(Color::Red))
            .block(Block::default().title("Error").borders(Borders::ALL));
        let area = Rect {
            x: body[1].x,
            y: body[1].y,
            width: body[1].width.min(50),
            height: 5,
        };
        frame.render_widget(err_box, area);
    }

    let help_idx = chunks.len() - 1;
    let help = Paragraph::new("Enter=fetch | ?=help | j/k=chapter | l=load | d=download | /=filter | g=jump | r/b lists | m=bookmark | t=theme | v=view | q=quit")
        .style(style);
    frame.render_widget(help, chunks[help_idx]);

    draw_overlay(frame, app);
}

fn draw_overlay(frame: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(70, 70, frame.area());
    match app.overlay {
        Overlay::None => {}
        Overlay::Help => {
            frame.render_widget(Clear, area);
            let text = vec![
                Line::from("Keyboard Help"),
                Line::from(""),
                Line::from("Enter: fetch URL"),
                Line::from("Arrow Left/Right/Home/End: edit URL cursor"),
                Line::from("Backspace/Delete: edit URL"),
                Line::from("j/k: select chapter"),
                Line::from("Up/Down/PageUp/PageDown/Home/End: scroll reader"),
                Line::from("l: load chapter text"),
                Line::from("d: download menu"),
                Line::from("/: filter chapters"),
                Line::from("g: jump to chapter number"),
                Line::from("r: recent URLs | b: bookmarks | m: bookmark current URL"),
                Line::from("t: theme toggle | v: compact/detailed view"),
                Line::from("e: toggle error details"),
                Line::from("Esc/Enter/q: close popup"),
            ];
            let p = Paragraph::new(text)
                .block(Block::default().title("Help").borders(Borders::ALL))
                .wrap(Wrap { trim: false });
            frame.render_widget(p, area);
        }
        Overlay::Download => {
            frame.render_widget(Clear, area);
            let p = Paragraph::new(vec![
                Line::from("Download Options"),
                Line::from(""),
                Line::from("1) Export story.json"),
                Line::from("2) Export chapters/*.txt bundle"),
                Line::from("3) Export selected chapter .txt"),
                Line::from(""),
                Line::from("Esc/q to cancel"),
            ])
            .block(Block::default().title("Download").borders(Borders::ALL));
            frame.render_widget(p, area);
        }
        Overlay::Recent => draw_pick_list(frame, area, "Recent URLs (press 1..9)", &app.recent_urls),
        Overlay::Bookmarks => draw_pick_list(frame, area, "Bookmarks (press 1..9)", &app.bookmarks),
    }
}

fn draw_pick_list(frame: &mut ratatui::Frame, area: Rect, title: &str, items: &[String]) {
    frame.render_widget(Clear, area);
    let mut lines = vec![Line::from(title), Line::from("")];
    if items.is_empty() {
        lines.push(Line::from("No entries."));
    } else {
        for (i, item) in items.iter().take(9).enumerate() {
            lines.push(Line::from(format!("{}. {}", i + 1, item)));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Esc/q to close"));

    let p = Paragraph::new(lines)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn persist_scroll(app: &mut App) {
    if let Some(idx) = app.selected_story_index() {
        app.chapter_scrolls.insert(idx, app.chapter_scroll);
    }
}

fn restore_scroll(app: &mut App) {
    if let Some(idx) = app.selected_story_index() {
        app.chapter_scroll = *app.chapter_scrolls.get(&idx).unwrap_or(&0);
    } else {
        app.chapter_scroll = 0;
    }
}

async fn fetch_story(url: &str) -> Result<Story, String> {
    let client = reqwest::Client::new();
    resolver::fetch_story_data_from_url(url, None, &client)
        .await
        .map_err(|e| e.to_string())
}

async fn load_selected_chapter_text(app: &App) -> Result<String, String> {
    let story = app
        .story
        .as_ref()
        .ok_or_else(|| "No story loaded".to_string())?;

    let idx = app
        .selected_story_index()
        .ok_or_else(|| "No chapter selected".to_string())?;

    let chapter = story
        .chapters
        .get(idx)
        .ok_or_else(|| "No chapter selected".to_string())?;

    if let Some(text) = &chapter.text {
        if !text.trim().is_empty() {
            return Ok(text.clone());
        }
    }

    let story_id = story
        .story_id
        .ok_or_else(|| "Story id missing in current story".to_string())?;
    let chapter_id = chapter.chapter_id.unwrap_or(0);
    let chapter_number = chapter.chapter_number.unwrap_or((idx + 1) as u32);

    let site = get_site(&story.site).map_err(|e| e.to_string())?;
    let client = reqwest::Client::new();
    let loaded = site
        .fetch_chapter(story_id, chapter_id, chapter_number, &client)
        .await
        .map_err(|e| e.to_string())?;

    Ok(loaded
        .text
        .unwrap_or_else(|| "(chapter has no text payload)".into()))
}

fn story_slug(story: &Story) -> String {
    story
        .story_name
        .clone()
        .unwrap_or_else(|| "story".into())
        .replace(['/', ' ', '\\'], "_")
}

fn download_story_json(story: &Option<Story>) -> Result<String, String> {
    let story = story
        .as_ref()
        .ok_or_else(|| "No story loaded to download".to_string())?;

    let file = format!("{}_story.json", story_slug(story));
    let json = serde_json::to_string_pretty(story).map_err(|e| e.to_string())?;
    std::fs::write(&file, json).map_err(|e| e.to_string())?;

    Ok(file)
}

fn download_chapter_txt_bundle(story: &Option<Story>) -> Result<String, String> {
    let story = story
        .as_ref()
        .ok_or_else(|| "No story loaded to download".to_string())?;

    let dir = format!("{}_chapters", story_slug(story));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    for (i, ch) in story.chapters.iter().enumerate() {
        let num = ch.chapter_number.unwrap_or((i + 1) as u32);
        let title = ch.title.clone().unwrap_or_else(|| format!("chapter_{}", num));
        let filename = format!("{}/{}_{}.txt", dir, num, title.replace(['/', ' ', '\\'], "_"));
        let text = ch.text.clone().unwrap_or_default();
        std::fs::write(filename, text).map_err(|e| e.to_string())?;
    }

    Ok(dir)
}

fn download_selected_chapter_txt(app: &App) -> Result<String, String> {
    let story = app
        .story
        .as_ref()
        .ok_or_else(|| "No story loaded to download".to_string())?;
    let idx = app
        .selected_story_index()
        .ok_or_else(|| "No selected chapter".to_string())?;
    let ch = story
        .chapters
        .get(idx)
        .ok_or_else(|| "No selected chapter".to_string())?;

    let num = ch.chapter_number.unwrap_or((idx + 1) as u32);
    let title = ch.title.clone().unwrap_or_else(|| format!("chapter_{}", num));
    let file = format!(
        "{}_{}_{}.txt",
        story_slug(story),
        num,
        title.replace(['/', ' ', '\\'], "_")
    );
    let text = if !app.chapter_text.is_empty() {
        app.chapter_text.clone()
    } else {
        ch.text.clone().unwrap_or_default()
    };
    std::fs::write(&file, text).map_err(|e| e.to_string())?;
    Ok(file)
}
