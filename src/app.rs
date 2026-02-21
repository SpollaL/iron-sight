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

pub struct ColumnStats {
    pub count: usize,
    pub min: String,
    pub max: String,
    pub mean: Option<f64>,
    pub median: Option<f64>,
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
    pub filters: Vec<(usize, String)>,
    pub filter_indices: Vec<usize>,
    pub filter_input: String,
    pub sort_column: Option<usize>,
    pub sort_direction: SortDirection,
    pub show_stats: bool,
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
            filter_input: String::new(),
            filter_indices: Vec::new(),
            filters: Vec::new(),
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            show_stats: false,
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
        let col = self.state.selected_column().unwrap_or(0);
        let input = self.filter_input.to_lowercase();
        self.filter_indices = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                self.filters
                    .iter()
                    .all(|(fc, fq)| r.get(*fc).map_or(false, |f| f.to_lowercase().contains(fq)))
                    && (input.is_empty()
                        || r.get(col)
                            .map_or(false, |f| f.to_lowercase().contains(&input)))
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
        if !self.filters.is_empty() || !self.filter_input.is_empty() {
            self.update_filter();
        }
    }

    pub fn compute_stats(&mut self, col: usize) -> ColumnStats {
        let values: Vec<&String> = self
            .records
            .iter()
            .filter_map(|r| r.get(col))
            .filter(|v| !v.is_empty())
            .collect();

        let numeric: Vec<f64> = self
            .records
            .iter()
            .filter_map(|r| r.get(col))
            .filter_map(|v| v.parse::<f64>().ok())
            .collect();

        let mean = if numeric.is_empty() {
            None
        } else {
            Some(numeric.iter().sum::<f64>() / numeric.len() as f64)
        };

        ColumnStats {
            count: self.records.len(),
            min: values
                .iter()
                .min()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            max: values
                .iter()
                .max()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            mean: mean,
            median: median(numeric),
        }
    }
}

fn median(mut array: Vec<f64>) -> Option<f64> {
    if array.is_empty() {
        return None;
    };
    array.sort_by(|a, b| a.total_cmp(b));
    let middle = array.len() / 2;
    if array.len() % 2 == 0 {
        Some((array[middle] + array[middle - 1]) / 2.0)
    } else {
        Some(array[middle])
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
        app.filters = vec![(0, "bob".to_string())];
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
