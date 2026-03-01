use polars::prelude::*;
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
    pub df: DataFrame,        // original data
    pub view: DataFrame,      // current filtered/sorted result
    pub headers: Vec<String>, // column names for display
    pub state: TableState,
    pub should_quit: bool,
    pub file_path: String,
    pub column_widths: Vec<u16>,
    pub mode: Mode,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_cursor: usize,
    pub filters: Vec<(usize, String)>,
    pub filter_input: String,
    pub sort_column: Option<usize>,
    pub sort_direction: SortDirection,
    pub show_stats: bool,
}

impl App {
    pub fn new(df: DataFrame, file_path: String) -> App {
        let headers: Vec<String> = df
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let column_count = headers.len();
        let view = df.clone();
        let mut app = App {
            df: df,
            view: view,
            headers: headers,
            state: TableState::default(),
            should_quit: false,
            file_path,
            column_widths: vec![DEFAULT_COLUMN_WIDTH; column_count],
            mode: Mode::Normal,
            search_query: String::new(),
            search_results: Vec::new(),
            search_cursor: 0,
            filter_input: String::new(),
            filters: Vec::new(),
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            show_stats: false,
        };
        if !app.df.is_empty() {
            app.state.select(Some(0));
            app.state.select_column(Some(0));
        }
        app
    }

    pub fn update_search(&mut self) {
        let current_column = self.state.selected_column().unwrap_or(0);
        let col_name = &self.headers[current_column];
        let query = self.search_query.to_lowercase();

        let series = self
            .view
            .column(col_name)
            .unwrap()
            .as_series()
            .unwrap()
            .cast(&DataType::String)
            .unwrap();

        self.search_results = series
            .str()
            .unwrap()
            .into_iter()
            .enumerate()
            .filter(|(_, val)| {
                val.map_or(false, |s| s.to_lowercase().contains(&query))
            })
            .map(|(i, _)| i)
            .collect();

        self.search_cursor = 0;
    }

    pub fn update_filter(&mut self) {
        let mut mask = lit(true);
        for (colidx, query) in &self.filters {
            let col_name = &self.headers[*colidx];
            mask = mask.and(
                col(col_name)
                    .cast(DataType::String)
                    .str()
                    .contains(lit(query.as_str()), false),
            );
        }
        if !self.filter_input.is_empty() {
            let col_name = &self.headers[self.state.selected_column().unwrap_or(0)];
            mask = mask.and(
                col(col_name)
                    .cast(DataType::String)
                    .str()
                    .contains(lit(self.filter_input.as_str()), false),
            )
        }
        let filtered = self
            .df
            .clone()
            .lazy()
            .filter(mask)
            .collect()
            .unwrap_or(self.df.clone());

        self.view = if let Some(sort_col) = self.sort_column {
            let col_name = &self.headers[sort_col];
            let opts = SortMultipleOptions::default().with_order_descending(matches!(
                self.sort_direction,
                SortDirection::Descending
            ));
            match filtered.sort([col_name], opts) {
                Ok(sorted) => sorted,
                Err(_) => filtered,
            }
        } else {
            filtered
        };
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
        let col_name = &self.headers[current_column];
        let opts = SortMultipleOptions::default().with_order_descending(matches!(
            self.sort_direction,
            SortDirection::Descending
        ));
        self.view = match self.view.sort([col_name], opts) {
            Ok(sorted) => sorted,
            Err(_) => self.view.clone(),
        };
    }

    pub fn compute_stats(&mut self, col: usize) -> ColumnStats {
        let col_name = &self.headers[col];
        let series = self.view.column(col_name).unwrap();

        let count = series.len();
        let min = series
            .min_reduce()
            .ok()
            .map(|s| s.value().to_string())
            .unwrap_or_default();
        let max = series
            .max_reduce()
            .ok()
            .map(|s| s.value().to_string())
            .unwrap_or_default();
        let mean = series.as_series().unwrap().mean();
        let median = series.as_series().unwrap().median();

        ColumnStats {
            count,
            min,
            max,
            mean,
            median,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> App {
        let df = df! {
            "name" => ["Alice", "Bob", "Charlie"],
            "age" => [30i64, 25, 35],
        }
        .unwrap();
        App::new(df, "test.csv".to_string())
    }

    fn get_str(app: &App, col: &str, row: usize) -> String {
        app.view
            .column(col)
            .unwrap()
            .as_series()
            .unwrap()
            .cast(&DataType::String)
            .unwrap()
            .str()
            .unwrap()
            .get(row)
            .unwrap_or("")
            .to_string()
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
        app.filters = vec![(0, "Bob".to_string())];
        app.update_filter();
        assert_eq!(app.view.height(), 1);
    }

    #[test]
    fn test_sort_by_column_ascending() {
        let mut app = make_app();
        app.state.select_column(Some(0));
        app.sort_by_column();
        assert_eq!(get_str(&app, "name", 0), "Alice");
        assert_eq!(get_str(&app, "name", 1), "Bob");
        assert_eq!(get_str(&app, "name", 2), "Charlie");
    }

    #[test]
    fn test_sort_by_column_toggles_descending() {
        let mut app = make_app();
        app.state.select_column(Some(0));
        app.sort_by_column();
        app.sort_by_column();
        assert_eq!(get_str(&app, "name", 0), "Charlie");
        assert_eq!(get_str(&app, "name", 1), "Bob");
        assert_eq!(get_str(&app, "name", 2), "Alice");
    }
}
