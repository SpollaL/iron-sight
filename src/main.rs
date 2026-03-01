mod app;
mod events;
mod ui;

use app::{App, Config};
use events::run_app;
use polars::prelude::*;
use std::{env, path::Path};

fn load_dataframe(file_path: &str) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "csv" => Ok(CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(file_path.into()))?
            .finish()?),
        "parquet" => Ok(ParquetReader::new(std::fs::File::open(file_path)?).finish()?),
        _ => Err(format!("Unsupported file format: .{}", ext).into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });

    let df = load_dataframe(&config.file_path).unwrap_or_else(|err| {
        eprintln!("Problem loading file: {}", err);
        std::process::exit(1);
    });

    let app = App::new(df, config.file_path);
    ratatui::run(|terminal| run_app(terminal, app))
}
