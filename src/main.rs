#[macro_use]
extern crate clap;

use anyhow::{Context, Result};
use clap::{App, Arg};
use germinate::Seed;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("germinate")
        .about("Template files using values from various sources")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("INPUT_FILE")
                .help("Template file to load and parse")
                .required(true),
        )
        .arg(
            Arg::with_name("output-file")
                .help("Path to write the output to")
                .short("o")
                .long("output-file")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    // Safe to unwrap the input file as it's a required argument
    let input = std::fs::read_to_string(matches.value_of("INPUT_FILE").unwrap())
        .context("Failed to read input file")?;

    let mut seed = Seed::new(&input);
    let output = seed.germinate().await?;

    // If no output file is given, write the output to stdout
    match matches.value_of("output-file").unwrap_or("-") {
        "-" => print!("{}", output),
        path => std::fs::write(path, output)?,
    };

    Ok(())
}
