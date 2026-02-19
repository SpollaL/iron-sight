mod app;
mod events;
mod ui;

use app::{App, Config};
use csv;
use events::run_app;
use std::env;

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
