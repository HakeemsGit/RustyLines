use std::error::Error;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use crate::{ContributorInfo, LanguageInfo, Stats};

// Define color constants for consistent styling
const TITLE_COLOR: Color = Color::Rgb(183, 65, 14);
const BORDER_COLOR: Color = Color::Rgb(139, 69, 19);
const TEXT_COLOR: Color = Color::Rgb(255, 160, 122);
const ERROR_COLOR: Color = Color::Red;

pub fn run(stats: Stats) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(stats);

    let res = run_app(&mut terminal, app);

    // Restore terminal state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

struct App {
    language_table: Vec<(String, LanguageInfo)>,
    contributor_table: Vec<(String, ContributorInfo)>,
    lang_state: TableState,
    contrib_state: TableState,
    focus_on_lang: bool,
    view_mode: bool,
}

impl App {
    fn new(stats: Stats) -> App {
        let mut language_table: Vec<_> = stats.languages.into_iter().collect();
        language_table.sort_by(|a, b| b.1.lines.cmp(&a.1.lines));

        let mut contributor_table: Vec<_> = stats.contributors.into_iter().collect();
        contributor_table.sort_by(|a, b| b.1.lines.cmp(&a.1.lines));

        App {
            language_table,
            contributor_table,
            lang_state: TableState::default(),
            contrib_state: TableState::default(),
            focus_on_lang: true,
            view_mode: false,
        }
    }

    fn next(&mut self) {
        if self.view_mode {
            return;
        }
        if self.focus_on_lang {
            let i = match self.lang_state.selected() {
                Some(i) => {
                    if i >= self.language_table.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.lang_state.select(Some(i));
        } else {
            let i = match self.contrib_state.selected() {
                Some(i) => {
                    if i >= self.contributor_table.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.contrib_state.select(Some(i));
        }
    }

    fn previous(&mut self) {
        if self.view_mode {
            return;
        }
        if self.focus_on_lang {
            let i = match self.lang_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.language_table.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => self.language_table.len() - 1,
            };
            self.lang_state.select(Some(i));
        } else {
            let i = match self.contrib_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.contributor_table.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => self.contributor_table.len() - 1,
            };
            self.contrib_state.select(Some(i));
        }
    }

    fn switch_focus(&mut self) {
        if self.view_mode {
            return;
        }
        self.focus_on_lang = !self.focus_on_lang;
    }

    fn toggle_view_mode(&mut self) {
        self.view_mode = !self.view_mode;
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        let size = terminal.get_frame().size();

        if size.width < 80 || size.height < 24 {
            terminal.draw(|f| {
                let msg = Paragraph::new("Please enlarge the terminal window.")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(ERROR_COLOR));
                f.render_widget(msg, f.size());
            })?;
        } else {
            terminal.draw(|f| ui(f, &mut app))?;
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Down => {
                        app.next();
                    }
                    KeyCode::Up => {
                        app.previous();
                    }
                    KeyCode::Tab => {
                        app.switch_focus();
                    }
                    KeyCode::Char('v') => {
                        app.toggle_view_mode();
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {
                    // No action needed; the loop will redraw
                }
                _ => {} // Ignore other events
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    if app.view_mode {
        render_detailed_view(f, size, app);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Tables
                    Constraint::Length(3), // Help
                ]
                .as_ref(),
            )
            .split(size);

        render_title(f, chunks[0]);
        render_tables(f, chunks[1], app);
        render_help(f, chunks[2]);
    }
}

fn render_title<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let title = Paragraph::new("RustyLines - Where Every Line Counts")
        .style(Style::default().fg(TITLE_COLOR).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR)),
        );
    f.render_widget(title, area);
}

fn render_tables<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    let tables_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    render_language_table(f, tables_chunks[0], app);
    render_contributor_table(f, tables_chunks[1], app);
}

fn render_language_table<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    let header_style = Style::default()
        .fg(TITLE_COLOR)
        .add_modifier(Modifier::BOLD);

    let header_cells = ["Languages", "Lines", "Files"]
        .iter()
        .map(|h| Cell::from(*h).style(header_style));

    let table_header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.language_table.iter().enumerate().map(|(i, (name, info))| {
        let cells = vec![
            Cell::from(name.clone()),
            Cell::from(format_integer(info.lines)),
            Cell::from(info.files.to_string()),
        ];
        let mut row = Row::new(cells).height(1);

        if app.focus_on_lang && Some(i) == app.lang_state.selected() && !app.view_mode {
            row = row.style(
                Style::default()
                    .bg(Color::Rgb(205, 92, 92))
                    .add_modifier(Modifier::BOLD),
            );
        }
        row
    });

    let table_block = Block::default()
        .title("File Info")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR));

    let table = Table::new(rows)
        .header(table_header)
        .block(table_block)
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .column_spacing(1)
        .style(Style::default().fg(TEXT_COLOR))
        .highlight_symbol(if app.view_mode { "" } else { ">> " });

    f.render_stateful_widget(table, area, &mut app.lang_state);
}

fn render_contributor_table<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    let header_style = Style::default()
        .fg(TITLE_COLOR)
        .add_modifier(Modifier::BOLD);

    let header_cells = ["Developers", "Lines", "Files"]
        .iter()
        .map(|h| Cell::from(*h).style(header_style));

    let table_header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app
        .contributor_table
        .iter()
        .enumerate()
        .map(|(i, (name, info))| {
            let cells = vec![
                Cell::from(name.clone()),
                Cell::from(format_integer(info.lines)),
                Cell::from(info.files.to_string()),
            ];
            let mut row = Row::new(cells).height(1);

            if !app.focus_on_lang && Some(i) == app.contrib_state.selected() && !app.view_mode {
                row = row.style(
                    Style::default()
                        .bg(Color::Rgb(205, 92, 92))
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        });

    let table_block = Block::default()
        .title("Contributors")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR));

    let table = Table::new(rows)
        .header(table_header)
        .block(table_block)
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .column_spacing(1)
        .style(Style::default().fg(TEXT_COLOR))
        .highlight_symbol(if app.view_mode { "" } else { ">> " });

    f.render_stateful_widget(table, area, &mut app.contrib_state);
}

fn render_help<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_text = Spans::from(vec![
        Span::styled("Use ", Style::default().fg(TEXT_COLOR)),
        Span::styled(
            "Up/Down",
            Style::default()
                .fg(TITLE_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" arrows to navigate, ", Style::default().fg(TEXT_COLOR)),
        Span::styled(
            "Tab",
            Style::default()
                .fg(TITLE_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to switch tables, ", Style::default().fg(TEXT_COLOR)),
        Span::styled(
            "'v'",
            Style::default()
                .fg(TITLE_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to view, ", Style::default().fg(TEXT_COLOR)),
        Span::styled(
            "'q'",
            Style::default()
                .fg(TITLE_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to quit", Style::default().fg(TEXT_COLOR)),
    ]);

    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR)),
        );
    f.render_widget(help, area);
}

fn render_detailed_view<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let title = if app.focus_on_lang {
        "Language Details"
    } else {
        "Contributor Details"
    };

    enum SelectedItem<'a> {
        Language(&'a (String, LanguageInfo)),
        Contributor(&'a (String, ContributorInfo)),
    }

    let selected_item = if app.focus_on_lang {
        app.lang_state
            .selected()
            .and_then(|i| app.language_table.get(i))
            .map(SelectedItem::Language)
    } else {
        app.contrib_state
            .selected()
            .and_then(|i| app.contributor_table.get(i))
            .map(SelectedItem::Contributor)
    };

    let content = if let Some(item) = selected_item {
        match item {
            SelectedItem::Language((name, info)) => format!(
                "Language: {}\nTotal Lines: {}\nFiles: {}",
                name,
                format_integer(info.lines),
                info.files
            ),
            SelectedItem::Contributor((name, info)) => format!(
                "Contributor: {}\nTotal Lines: {}\nFiles: {}",
                name,
                format_integer(info.lines),
                info.files
            ),
        }
    } else {
        "No item selected.".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR));

    let paragraph = Paragraph::new(content)
        .alignment(Alignment::Left)
        .block(block)
        .style(Style::default().fg(TEXT_COLOR));

    f.render_widget(paragraph, area);
}

fn format_integer(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::new();

    for (i, &b) in bytes.iter().enumerate() {
        result.push(b as char);
        if (len - i - 1) % 3 == 0 && i != len - 1 {
            result.push(',');
        }
    }
    result
}
