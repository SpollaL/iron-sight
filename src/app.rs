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

pub struct App {
    pub headers: Vec<String>,
    pub records: Vec<Vec<String>>,
    pub state: TableState,
    pub should_quit: bool,
    pub filepath: String,
}

impl App {
    pub fn new(headers: Vec<String>, records: Vec<Vec<String>>, filepath: String) -> App {
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
