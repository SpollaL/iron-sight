use std::env;
use csv;

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

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });
    csv::Reader::from_path(config.file_path).unwrap_or_else(|err| {
        eprintln!("Problem reading the file: {}", err);
        std::process::exit(1);
    }).into_records().for_each(|result| {
        match result {
            Ok(record) => println!("{:?}", record),
            Err(err) => eprintln!("Error reading record: {}", err),
        }
    });
}
