use ratatui::widgets::TableState;

const DEFAULT_COLUMN_WIDTH: u16 = 15;

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
        Ok(Config { file_path })
    }
}

#[derive(Debug)]
pub enum Mode {
    Search,
    Normal,
    Filter,
}

#[derive(Debug)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct App {
    pub headers: Vec<String>,
    pub records: Vec<Vec<String>>,
    pub state: TableState,
    pub should_quit: bool,
    pub file_path: String,
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
    pub fn new(headers: Vec<String>, records: Vec<Vec<String>>, file_path: String) -> App {
        let column_count = headers.len();
        let mut app = App {
            headers,
            records,
            state: TableState::default(),
            should_quit: false,
            file_path,
            column_widths: vec![DEFAULT_COLUMN_WIDTH; column_count],
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
        self.search_results = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.get(current_column)
                    .map_or(false, |f| f.to_lowercase().contains(&query))
            })
            .map(|(i, _)| i)
            .collect();
        self.search_cursor = 0;
    }

    pub fn update_filter(&mut self) {
        let query = self.filter_query.to_lowercase();
        self.filter_indices = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.get(self.filter_column.unwrap_or(0))
                    .map_or(false, |f| f.to_lowercase().contains(&query))
            })
            .map(|(i, _)| i)
            .collect();
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
        }
        self.records.sort_by(|a, b| match self.sort_direction {
            SortDirection::Ascending => a.get(current_column).cmp(&b.get(current_column)),
            SortDirection::Descending => b.get(current_column).cmp(&a.get(current_column)),
        });
        if !self.filter_query.is_empty() {
            self.update_filter();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> App {
        let headers = vec!["name".to_string(), "age".to_string()];
        let records = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
            vec!["Charlie".to_string(), "35".to_string()],
        ];
        App::new(headers, records, "test.csv".to_string())
    }

    #[test]
    fn test_update_search_finds_matches() {
        let mut app = make_app();
        app.search_query = "alice".to_string();
        app.update_search();
        assert_eq!(app.search_results, vec![0]);
    }

    #[test]
    fn test_update_search_case_insensitive() {
        let mut app = make_app();
        app.search_query = "ALICE".to_string();
        app.update_search();
        assert_eq!(app.search_results, vec![0]);
    }

    #[test]
    fn test_update_search_no_matches() {
        let mut app = make_app();
        app.search_query = "xyz".to_string();
        app.update_search();
        assert!(app.search_results.is_empty());
    }

    #[test]
    fn test_update_filter_finds_matches() {
        let mut app = make_app();
        app.filter_column = Some(0);
        app.filter_query = "bob".to_string();
        app.update_filter();
        assert_eq!(app.filter_indices, vec![1]);
    }

    #[test]
    fn test_sort_by_column_ascending() {
        let mut app = make_app();
        app.state.select_column(Some(0));
        app.sort_by_column();
        assert_eq!(app.records[0][0], "Alice");
        assert_eq!(app.records[1][0], "Bob");
        assert_eq!(app.records[2][0], "Charlie");
    }

    #[test]
    fn test_sort_by_column_toggles_descending() {
        let mut app = make_app();
        app.state.select_column(Some(0));
        app.sort_by_column();
        app.sort_by_column();
        assert_eq!(app.records[0][0], "Charlie");
        assert_eq!(app.records[1][0], "Bob");
        assert_eq!(app.records[2][0], "Alice");
    }
}
