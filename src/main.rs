use crossterm::event;
use csv;
use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize, Color, Modifier};
use ratatui::widgets::{Cell, Row, Table, TableState};
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
    shoudl_quit: bool,
}

impl App {
    fn new(headers: Vec<String>, records: Vec<Vec<String>>) -> App {
        let mut app = App {
            headers,
            records,
            state: TableState::default(),
            shoudl_quit: false,
        };
        app.state.select(Some(0));
        app
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.records.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.records.len() - 1 
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
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
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(table, frame.area(), &mut app.state);
}

fn run_app(
    temrinal: &mut ratatui::DefaultTerminal,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    while !app.shoudl_quit {
        temrinal.draw(|frame| ui(frame, &mut app))?;

        if let event::Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Char('q') => app.shoudl_quit = true,
                event::KeyCode::Down => app.next(),
                event::KeyCode::Up => app.previous(),
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
    let app = App::new(headers, data.collect());
    ratatui::run(|terminal| run_app(terminal, app))
}
