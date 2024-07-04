use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::{Duration, Instant}};
use sysinfo::{ProcessExt, System, SystemExt};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState, Paragraph},
    text::{Span, Spans},
    Frame, Terminal,
};

struct App {
    sys: System,
    table_state: TableState,
}

impl App {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys,
            table_state: TableState::default(),
        }
    }

    fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.sys.processes().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.sys.processes().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn kill(&mut self) {
        if let Some(i) = self.table_state.selected() {
            let pid = self.sys.processes().keys().nth(i).cloned();
            if let Some(pid) = pid {
                self.sys.process(pid).map(|p| p.kill());
            }
        }
    }

    fn refresh(&mut self) {
        self.sys.refresh_all();
    }
}

fn main() -> Result<(), io::Error> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Char('k') => app.kill(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.refresh();
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    let help_message = Paragraph::new(Spans::from(vec![
        Span::raw("q: Quit | "),
        Span::raw("↑/↓: Move | "),
        Span::raw("k: Kill Process"),
    ]))
    .style(Style::default().fg(Color::LightCyan))
    .block(Block::default().borders(Borders::ALL).title("Help"));

    f.render_widget(help_message, chunks[0]);

    let processes: Vec<_> = app.sys.processes().values().collect();
    let rows = processes.iter().map(|process| {
        Row::new(vec![
            process.pid().to_string(),
            process.name().to_string(),
            format!("{:.2}", process.cpu_usage()),
            format!("{:.2} MB", process.memory() as f64 / 1024.0 / 1024.0), // Corrected to MB
        ])
    });

    let table = Table::new(rows)
        .header(Row::new(vec![
            "PID", "Name", "CPU %", "Memory (MB)"
        ]).style(Style::default().fg(Color::Yellow)).bottom_margin(1))
        .block(Block::default().borders(Borders::ALL).title("Processes"))
        .widths(&[
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(20),
        ])
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, chunks[1], &mut app.table_state);
}

