use crossterm::event;
use csv;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState};
use ratatui::Frame;
use std::env;

struct Config {
    file_path: String,
}
impl Config {
    fn new(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();
        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Please provide a valid CSV file path"),
        };

        Ok(Config {
            file_path: file_path,
        })
    }
}

struct App {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
    state: TableState,
    should_quit: bool,
    filepath: String,
}

impl App {
    fn new(headers: Vec<String>, records: Vec<Vec<String>>, filepath: String) -> App {
        let mut app = App {
            headers,
            records,
            state: TableState::default(),
            should_quit: false,
            filepath: filepath,
        };
        if !app.records.is_empty() {
            app.state.select(Some(0));
            app.state.select_column(Some(0));
        }
        app
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    let header_cells = Row::new(app.headers.iter().map(|header| Cell::from(header.as_str())));
    let rows = app
        .records
        .iter()
        .map(|record| Row::new(record.iter().map(|field| Cell::from(field.as_str()))))
        .collect::<Vec<Row>>();
    let column_count = app.records.first().map_or(0, |record| record.len());
    let widths = vec![Constraint::Length(15); column_count];
    let table = Table::new(rows, widths)
        .header(header_cells.bold().bottom_margin(1))
        .block(
            ratatui::widgets::Block::default()
                .title("CSV Viewer")
                .borders(ratatui::widgets::Borders::ALL),
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray))
        .column_highlight_style(Style::default().bg(Color::DarkGray))
        .cell_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());
    let bar = format!(
        " Row {}/{} | Col {}/{} | {} ",
        app.state.selected().map_or(0, |i| i + 1),
        app.records.len(),
        app.state.selected_column().map_or(0, |i| i + 1),
        app.headers.len(),
        app.filepath
    );
    let bar = Paragraph::new(bar).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_stateful_widget(table, chunks[0], &mut app.state);
    frame.render_widget(bar, chunks[1]);
}

fn run_app(
    temrinal: &mut ratatui::DefaultTerminal,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    while !app.should_quit {
        temrinal.draw(|frame| ui(frame, &mut app))?;

        if let event::Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Char('q') => app.should_quit = true,
                event::KeyCode::Down => app.state.select_next(),
                event::KeyCode::Up => app.state.select_previous(),
                event::KeyCode::Left => app.state.select_previous_column(),
                event::KeyCode::Right => app.state.select_next_column(),
                event::KeyCode::Char('j') => app.state.select_next(),
                event::KeyCode::Char('k') => app.state.select_previous(),
                event::KeyCode::Char('h') => app.state.select_previous_column(),
                event::KeyCode::Char('l') => app.state.select_next_column(),
                event::KeyCode::Char('g') => app.state.select_first(),
                event::KeyCode::Char('G') => app.state.select_last(),
                event::KeyCode::PageDown => app.state.scroll_down_by(20),
                event::KeyCode::PageUp => app.state.scroll_up_by(20),
                event::KeyCode::Home => app.state.select_first(),
                event::KeyCode::End => app.state.select_last(),
                _ => {}
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });
    let mut reader = csv::Reader::from_path(&config.file_path).unwrap_or_else(|err| {
        eprintln!("Problem reading the file: {}", err);
        std::process::exit(1);
    });
    let headers = reader
        .headers()
        .unwrap_or_else(|err| {
            eprintln!("Problem reading the CSV headers: {}", err);
            std::process::exit(1);
        })
        .iter()
        .map(|header| header.to_string())
        .collect::<Vec<String>>();

    let data = reader.into_records().map(|result| {
        result
            .unwrap_or_else(|err| {
                eprintln!("Problem parsing the CSV data: {}", err);
                std::process::exit(1);
            })
            .iter()
            .map(|field| field.to_string())
            .collect::<Vec<String>>()
    });
    let app = App::new(headers, data.collect(), config.file_path);
    ratatui::run(|terminal| run_app(terminal, app))
}
