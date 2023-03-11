use std::io::Write;
use std::path::Path;

use clap::{Arg, Command};
use log::{info, error};
use mzsniffer::mzml::MzMLReader;
use mzsniffer::search::{search, PolymerResults};
use tokio::fs::File;
use tokio::io::BufReader;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .parse_env(
            env_logger::Env::default()
                .filter_or("MZSNIFFER_LOG", "error,mzsniffer=info")
        )
        .init();


    let matches = Command::new("mzsniffer")
        .version(clap::crate_version!())
        .author("William E. Fondrie <fondriew@gmail.com>")
        .about("\u{1F9A8} mzsniffer \u{1F443} - Detect polymer conminants in mass spectra.")
        .arg(Arg::new("mzml_paths").help("The mzML file(s) to analyze.").num_args(1..))
        .arg(
            Arg::new("tol")
                .short('t')
                .long("tolerance")
                .help("The precursor mass tolerance.")
                .default_value("10")
                .value_parser(clap::value_parser!(f64))
        )
        .arg(
            Arg::new("unit")
                .short('d')
                .long("use-da")
                .help("Use Da instead of ppm as the precursor mass tolerance unit.")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .help(
                    "Specify an output format to be sent to stdout. \
                     Must be one of 'json' or 'pickle'."
                )
        )
        .help_template(
            "{usage-heading} {usage}\n\n\
             {about-with-newline}\n\
             Written by {author-with-newline}Version {version}\n\n\
             {all-args}{after-help}",
        )
        .get_matches();

    let mut mzml_paths: Vec<String> = Vec::new();
    if let Some(f) = matches.get_many::<String>("mzml_paths") {
        mzml_paths = f.into_iter().map(|p| p.into()).collect();
    } else {
        error!("An mzML file must be provided.")
    }
    let unit = match matches.get_one::<bool>("unit") {
        Some(true) => "da",
        Some(false) => "ppm",
        None => unreachable!("This shouldnt happen.")
    };
    let tol = matches.get_one::<f64>("tol").unwrap();

    let out_format = matches.get_one::<String>("format");
    match out_format {
        Some(txt) => {
            match txt.to_lowercase().as_str() {
                "json" | "pickle" => {},
                _ => error!("Unrecognized output format.")
            }
        },
        None => {},
    }

    // Actually do stuff:
    for mzml_file in mzml_paths.into_iter() {
        run(mzml_file, *tol, unit.to_string()).await?;
    }

    info!("DONE!");
    let total_time = std::time::Instant::now() - start;
    info!("Elapsed time: {:2}s", total_time.as_secs());
    Ok(())
}


async fn run(
    mzml_file: String,
    tol: f64,
    unit: String
) -> anyhow::Result<PolymerResults> {
    info!("Reading {}...", &mzml_file);
    let start = std::time::Instant::now();
    let mzml_buf = File::open(mzml_file.as_str()).await?;
    let mzml_buf = BufReader::new(mzml_buf);
    let spectra = MzMLReader::new().parse(mzml_buf).await?;
    let total_time = std::time::Instant::now() - start;
    info!(" - Read time: {:2}s", total_time.as_secs());

    let start = std::time::Instant::now();
    info!("Extracting MS1 signals of polymer contaminants...");
    let mzml_file = Path::new(mzml_file.as_str())
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let results = search(mzml_file, spectra, &tol, unit.as_str())?;
    let total_time = std::time::Instant::now() - start;
    info!(" - Extraction time: {:2}s ", total_time.as_secs());
    info!("{}", "+".repeat(36));
    info!("Polymer                         %TIC");
    info!("{}", "+".repeat(36));

    // Print a brief report to stderr:
    for poly in results.polymers.clone().into_iter() {
        info!(
            "{:26}  {:>8.4}",
            &poly.name,
            &100. * &poly.total / &results.total,
        );
    }
    info!("{}", "+".repeat(36));
    info!("");

    Ok(results)
}
