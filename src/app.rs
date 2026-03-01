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
    PlotPickX,
    Plot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlotType {
    Line,
    Bar,
}

#[derive(Debug)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggFunc {
    Sum,
    Mean,
    Count,
    Min,
    Max,
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
    pub show_help: bool,
    pub groupby_keys: Vec<usize>,
    pub groupby_aggs: Vec<(usize, AggFunc)>,
    pub groupby_active: bool,
    pub saved_headers: Vec<String>,
    pub saved_column_widths: Vec<u16>,
    pub plot_y_col: Option<usize>,
    pub plot_x_col: Option<usize>,
    pub plot_type: PlotType,
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
            show_help: false,
            groupby_keys: Vec::new(),
            groupby_aggs: Vec::new(),
            groupby_active: false,
            saved_headers: Vec::new(),
            saved_column_widths: Vec::new(),
            plot_y_col: None,
            plot_x_col: None,
            plot_type: PlotType::Line,
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
            .filter(|(_, val)| val.map_or(false, |s| s.to_lowercase().contains(&query)))
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
            let opts = SortMultipleOptions::default()
                .with_order_descending(matches!(self.sort_direction, SortDirection::Descending));
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
        let opts = SortMultipleOptions::default()
            .with_order_descending(matches!(self.sort_direction, SortDirection::Descending));
        self.view = match self.view.sort([col_name], opts) {
            Ok(sorted) => sorted,
            Err(_) => self.view.clone(),
        };
    }

    pub fn autofit_selected_column(&mut self) {
        if let Some(col_idx) = self.state.selected_column() {
            let label = self.header_label(col_idx);
            let header_width = label.chars().count() as u16;
            let col_name = self.headers[col_idx].clone();
            let max_data = self
                .view
                .column(&col_name)
                .ok()
                .and_then(|col| {
                    let cast = col.as_series()?.cast(&DataType::String).ok()?;
                    let max = cast
                        .str()
                        .ok()?
                        .into_iter()
                        .filter_map(|v| v)
                        .map(|s: &str| s.chars().count())
                        .max()
                        .map(|n| n as u16);
                    max
                })
                .unwrap_or(0);
            self.column_widths[col_idx] = max_data.max(header_width);
        }
    }

    pub fn header_label(&self, col_idx: usize) -> String {
        let base = &self.headers[col_idx];
        let label = if self.sort_column == Some(col_idx) {
            let dir = if matches!(self.sort_direction, SortDirection::Descending) {
                "▼"
            } else {
                "▲"
            };
            format!("{} {}", base, dir)
        } else {
            base.clone()
        };
        if self.groupby_keys.contains(&col_idx) {
            format!("{} [K]", label)
        } else if let Some((_, func)) = self.groupby_aggs.iter().find(|(j, _)| *j == col_idx) {
            let sym = match func {
                AggFunc::Sum => "Σ",
                AggFunc::Mean => "μ",
                AggFunc::Count => "#",
                AggFunc::Min => "↓",
                AggFunc::Max => "↑",
            };
            format!("{} [{}]", label, sym)
        } else {
            label
        }
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

    pub fn toggle_groupby_key(&mut self) {
        let col = self.state.selected_column().unwrap_or(0);
        if let Some(pos) = self.groupby_keys.iter().position(|&k| k == col) {
            self.groupby_keys.remove(pos);
        } else {
            self.groupby_keys.push(col);
            self.groupby_aggs.retain(|(i, _)| *i != col);
        }
    }

    pub fn cycle_groupby_agg(&mut self) {
        let col = self.state.selected_column().unwrap_or(0);
        if self.groupby_keys.contains(&col) {
            return;
        };
        if let Some(pos) = self.groupby_aggs.iter().position(|(i, _)| *i == col) {
            let next = match self.groupby_aggs[pos].1 {
                AggFunc::Sum => Some(AggFunc::Mean),
                AggFunc::Mean => Some(AggFunc::Count),
                AggFunc::Count => Some(AggFunc::Min),
                AggFunc::Min => Some(AggFunc::Max),
                AggFunc::Max => None,
            };
            self.groupby_aggs.remove(pos);
            if let Some(func) = next {
                self.groupby_aggs.push((col, func));
            };
        } else {
            self.groupby_aggs.push((col, AggFunc::Sum));
        }
    }
    pub fn apply_groupby(&mut self) {
        if self.groupby_keys.is_empty() || self.groupby_aggs.is_empty() {
            return;
        }
        let key_exprs: Vec<Expr> = self
            .groupby_keys
            .iter()
            .map(|&i| col(&self.headers[i]))
            .collect();
        let agg_exprs: Vec<Expr> = self
            .groupby_aggs
            .iter()
            .map(|(i, func)| {
                let name = &self.headers[*i];
                match func {
                    AggFunc::Sum => col(name).sum().alias(&format!("{}_sum", name)),
                    AggFunc::Mean => col(name).mean().alias(&format!("{}_mean", name)),
                    AggFunc::Count => col(name).count().alias(&format!("{}_count", name)),
                    AggFunc::Min => col(name).min().alias(&format!("{}_min", name)),
                    AggFunc::Max => col(name).max().alias(&format!("{}_max", name)),
                }
            })
            .collect();
        let first_key = self.headers[self.groupby_keys[0]].clone();
        let result = self
            .view
            .clone()
            .lazy()
            .group_by(key_exprs)
            .agg(agg_exprs)
            .sort([&first_key], SortMultipleOptions::default())
            .collect();
        if let Ok(df) = result {
            self.saved_headers = self.headers.clone();
            self.saved_column_widths = self.column_widths.clone();
            self.headers = df
                .get_column_names()
                .iter()
                .map(|s| s.to_string())
                .collect();
            self.column_widths = vec![DEFAULT_COLUMN_WIDTH; df.width()];
            self.sort_column = None;
            self.search_results = Vec::new();
            self.search_cursor = 0;
            self.view = df;
            self.groupby_active = true;
            self.state.select(Some(0));
            self.state.select_column(Some(0));
        }
    }

    pub fn plot_type_label(&self) -> &str {
        match self.plot_type {
            PlotType::Line => "Line",
            PlotType::Bar => "Bar",
        }
    }

    pub fn clear_groupby(&mut self) {
        self.headers = self.saved_headers.clone();
        self.column_widths = self.saved_column_widths.clone();
        self.groupby_keys = Vec::new();
        self.groupby_aggs = Vec::new();
        self.groupby_active = false;
        self.sort_column = None;
        self.search_results = Vec::new();
        self.search_cursor = 0;
        self.update_filter();
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
    fn test_autofit_uses_data_width() {
        let mut app = make_app();
        app.state.select_column(Some(0));
        app.autofit_selected_column();
        // "name" col: max("Alice"=5, "Bob"=3, "Charlie"=7) = 7, header "name" = 4
        assert_eq!(app.column_widths[0], 7);
    }

    #[test]
    fn test_autofit_accounts_for_groupby_marker() {
        let mut app = make_app();
        app.state.select_column(Some(0));
        app.toggle_groupby_key(); // adds [K] to header: "name [K]" = 8 chars
        app.autofit_selected_column();
        // header "name [K]" = 8 chars > data max 7 → width should be 8
        assert_eq!(app.column_widths[0], 8);
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

#[cfg(test)]
mod plot_tests {
    use super::*;

    #[test]
    fn test_extract_plot_basic() {
        let df = df! {
            "x" => [1i32, 2i32, 3i32],
            "y" => [10i32, 20i32, 30i32],
        }.unwrap();
        let app = App::new(df, "test.csv".to_string());
        let (data, x_is_categorical) = crate::ui::extract_plot_data_pub(&app, 0, 1);
        assert!(!data.is_empty(), "both numeric: data should not be empty");
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], (1.0, 10.0));
        assert!(!x_is_categorical, "numeric x: not categorical");
    }

    #[test]
    fn test_extract_plot_string_x() {
        let df = df! {
            "name" => ["alpha", "beta", "gamma"],
            "qty"  => [10i32, 20i32, 30i32],
        }.unwrap();
        let app = App::new(df, "test.csv".to_string());
        let (data, x_is_categorical) = crate::ui::extract_plot_data_pub(&app, 0, 1);
        assert!(!data.is_empty(), "string x: should use row index");
        assert_eq!(data[0], (0.0, 10.0));
        assert!(x_is_categorical, "string x: should be categorical");
    }
}
