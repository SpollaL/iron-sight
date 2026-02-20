use std::vec;

use ratatui::widgets::TableState;

pub struct Config {
    pub file_path: String,
}
impl Config {
    pub fn new(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
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

pub enum Mode {
    Search,
    Normal,
}

pub struct App {
    pub headers: Vec<String>,
    pub records: Vec<Vec<String>>,
    pub state: TableState,
    pub should_quit: bool,
    pub filepath: String,
    pub column_widths: Vec<u16>,
    pub mode: Mode,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_cursor: usize,
}

impl App {
    pub fn new(headers: Vec<String>, records: Vec<Vec<String>>, filepath: String) -> App {
        let column_count = headers.len();
        let mut app = App {
            headers,
            records,
            state: TableState::default(),
            should_quit: false,
            filepath: filepath,
            column_widths: vec![15; column_count],
            mode: Mode::Normal,
            search_query: String::new(),
            search_results: Vec::new(),
            search_cursor: 0,
        };
        if !app.records.is_empty() {
            app.state.select(Some(0));
            app.state.select_column(Some(0));
        }
        app
    }
    pub fn update_search(&mut self) {
        let current_column = self.state.selected_column().unwrap_or(0);
        let query = self.search_query.to_lowercase();
        let search_results: Vec<usize> = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.get(current_column)
                    .map_or(false, |f| f.to_lowercase().contains(&query))
            })
            .map(|(i, _)| i)
            .collect();
        self.search_results = search_results;
        self.search_cursor = 0;
    }
}
