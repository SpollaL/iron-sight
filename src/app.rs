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
    Filter,
}

pub enum SortDirection {
    Ascending,
    Descending,
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
    pub filter_query: String,
    pub filter_indices: Vec<usize>,
    pub filter_column: Option<usize>,
    pub sort_column: Option<usize>,
    pub sort_direction: SortDirection,
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
            filter_query: String::new(),
            filter_indices: Vec::new(),
            filter_column: None,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
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

    pub fn update_filter(&mut self) {
        let query = self.filter_query.to_lowercase();
        let filter_indices: Vec<usize> = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.get(self.filter_column.unwrap_or(0))
                    .map_or(false, |f| f.to_lowercase().contains(&query))
            })
            .map(|(i, _)| i)
            .collect();
        self.filter_indices = filter_indices;
        if !self.search_results.is_empty() {
            self.update_search();
        }
    }

    pub fn sort_by_column(&mut self) {
        let current_column = self.state.selected_column().unwrap_or(0);
        if self.sort_column == Some(current_column) {
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_column = Some(current_column);
            self.sort_direction = SortDirection::Ascending;
        };
        self.records.sort_by(|a, b| match self.sort_direction {
                SortDirection::Ascending => a.get(current_column).cmp(&b.get(current_column)),
                SortDirection::Descending => b.get(current_column).cmp(&a.get(current_column)),
        });
        if !self.filter_query.is_empty() {
            self.update_filter();
        }
    }
}
