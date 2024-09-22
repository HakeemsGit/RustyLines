use std::error::Error;
use clap::{App, Arg};
use crate::{get_stats, tui};

pub fn execute() -> Result<(), Box<dyn Error>> {
    let matches = App::new("RustyLines")
        .version("1.0")
        .author("HakeemsGit")
        .about("Count lines of code in a directory")
        .arg(
            Arg::with_name("path")
                .help("The path to analyze")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = matches
        .value_of("path")
        .ok_or("Path argument is required")?;

    let stats = get_stats(path)?;
    tui::run(stats)
}
