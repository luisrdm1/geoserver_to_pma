use geoserver_to_pma as gs;
use clap::Parser;

/// Simple program to convert GeoServer files into PMA databases
#[derive(Parser, Debug)]
#[command(author,
version,
about,
long_about = None)]
struct Args {
    /// Input file path of a CSV file containing GeoServer info
    #[arg(short, long, value_name = "INPUT")]
    input_path: Option<std::path::PathBuf>,

    /// Maximum latitude to include in the resulting file
    #[arg(short, long, allow_hyphen_values = true, default_value_t = -23.0)]
    max_lat: f64,

    /// Output optional file path of a CSV file containing GeoServer info
    #[arg(short, long, value_name = "OUTPUT")]
    output_path: Option<std::path::PathBuf>,
}

fn main() {
    let args = Args::parse();

    let final_path = match args.input_path {
        None => {
            eprintln!("You didn't give a valid file path. Aborting.");
            std::process::exit(1);
        }
        Some(path) => path,
    };

    let dest_ext = final_path.with_extension("txt");

    let data = gs::AiswebAirportJSON::new(&final_path);

    data.write_pma_txt(&dest_ext);
}