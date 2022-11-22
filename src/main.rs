use std::path::PathBuf;

use clap::Parser;
use geo_pma as gs;

/// Simple program to convert GeoServer files into PMA databases
#[derive(Parser, Debug)]
#[command(author,
version,
about,
long_about = None)]
struct Args {
    /// Input file path of a JSON file containing GeoServer info
    #[arg(short, long, value_name = "INPUT")]
    input_path: std::path::PathBuf,

    /// Maximum latitude to include in the resulting file
    #[arg(short, long, allow_negative_numbers = true, default_value_t = -23.0)]
    max_lat: f64,

    /// Optional output file path of a "txt" in PMA format
    #[arg(short, long, value_name = "OUTPUT")]
    output_path: Option<std::path::PathBuf>,

    /// Offline usage, by indicating the path of the input files
    #[arg(short, long, value_name = "PATH")]
    path: Option<Vec<std::path::PathBuf>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let options = ["waypoint", "ndb", "vor", "runway_v2", "rwydirection", "airport"];

    let mut structs = Vec::new();

    for option in options {
        let target = option;
    
        let aisweb_url = format!("https://geoaisweb.decea.mil.br/geoserver/ICA/ows?service=WFS&version=1.0.0&request=GetFeature&typeName=ICA:{target}&outputFormat=application%2Fjson");
    
        // waypoint, ndb, vor, runway_v2, rwydirection (cabeceiras), airport
    
        let response = reqwest::get(aisweb_url).await?;
    
        let deserial: gs::AiswebJSON = response.json().await?;

        structs.push(deserial);
    }
    
    let args = Args::parse();

    let input_file = args.input_path;

    if let Some(path) = args.output_path {
        let output_ext = path;
        run(&input_file, &output_ext)
    } else {
        let output_ext = input_file.with_extension("txt");
        run(&input_file, &output_ext)
    }

    Ok(())
}

fn run(input_file: &PathBuf, output_ext: &PathBuf) -> () {
    let data = gs::AiswebJSON::new(&input_file);

    data.decide(&output_ext);
}