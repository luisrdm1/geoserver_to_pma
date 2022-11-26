use clap::Parser;
use geo_pma as gs;
use gs::{AiswebJSON, CompleteThresholds, Options, WritingDB};
use serde::de::DeserializeOwned;
use std::path::PathBuf;

/// Simple program to convert GeoServer files into PMA databases
#[derive(Parser, Debug)]
#[command(author,
version,
about,
long_about = None)]
struct Args {
    /// Optional output directory to save the generated txt files (relative or full path)
    #[arg(short, long, value_name = "PATH")]
    output_directory: Option<std::path::PathBuf>,
    // /// Maximum latitude to include in the resulting file
    // #[arg(short, long, allow_negative_numbers = true, default_value_t = -23.0)]
    // max_lat: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let save_dir = match args.output_directory {
        Some(path) => path,
        None => PathBuf::new(),
    };

    let waypoints: AiswebJSON<gs::Fixes> = request_deserial(Options::Fixes).await?;
    let ndbs: AiswebJSON<gs::NDB> = request_deserial(Options::NDB).await?;
    let vors: AiswebJSON<gs::Vor> = request_deserial(Options::VOR).await?;
    let runways: AiswebJSON<gs::RunwayV2> = request_deserial(Options::RunwayV2).await?;
    let thresholds: AiswebJSON<gs::Thresholds> = request_deserial(Options::Thresholds).await?;
    let airports: AiswebJSON<gs::Airport> = request_deserial(Options::Airport).await?;

    let mut complete_thresholds = vec![];

    for threshold in thresholds.features.iter() {
        for airport in airports.features.iter() {
            for runway in runways.features.iter() {
                if threshold.properties.runway_pk == runway.properties.runway_pk
                    && runway.properties.airport_pk == airport.properties.airport_pk.unwrap_or(0)
                {
                    complete_thresholds.push(CompleteThresholds::new(
                        &airport.properties,
                        &runway.properties,
                        &threshold.properties,
                    ));
                }
            }
        }
    }

    drop(thresholds);
    drop(runways);

    match waypoints.write_pma_txt(Options::Fixes, &save_dir) {
        Ok(_) => println!("Created waypoints DB."),
        Err(e) => eprintln!("Somenthing went wrong: {}", e),
    }
    match ndbs.write_pma_txt(Options::NDB, &save_dir) {
        Ok(_) => println!("Created NDB DB."),
        Err(e) => eprintln!("Somenthing went wrong: {}", e),
    }
    match vors.write_pma_txt(Options::VOR, &save_dir) {
        Ok(_) => println!("Created VOR DB."),
        Err(e) => eprintln!("Somenthing went wrong: {}", e),
    }
    match airports.write_pma_txt(Options::Airport, &save_dir) {
        Ok(_) => println!("Created Airport DB."),
        Err(e) => eprintln!("Somenthing went wrong: {}", e),
    }
    match complete_thresholds.write_pma_txt(Options::CompleteThresholds, &save_dir) {
        Ok(_) => println!("Created Thresholds DB."),
        Err(e) => eprintln!("Somenthing went wrong: {}", e),
    }

    Ok(())
}

async fn request_deserial<T: DeserializeOwned>(
    target: Options,
) -> Result<AiswebJSON<T>, Box<dyn std::error::Error>> {
    let aisweb_url = format!(
        "https://geoaisweb.decea.mil.br/geoserver/ICA/ows?service=WFS&version=1.0.0&request=GetFeature&typeName=ICA:{target}&outputFormat=application%2Fjson");

    let response = reqwest::get(aisweb_url).await?;

    let deserialized: gs::AiswebJSON<T> = response.json().await?;

    Ok(deserialized)
}
