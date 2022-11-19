use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, path};

#[derive(Deserialize, Debug)]
pub struct AiswebAirportJSON {
    features: Vec<Airport>,
}

#[derive(Deserialize, Debug)]
struct Airport {
    id: String,
    properties: AirportProperties,
}

#[derive(Deserialize, Debug)]
struct AirportProperties {
    localidade_id: String,
    nome: String,
    opr: String,
    latitude_dec: f64,
    longitude_dec: f64,
    elevacao: f64,
}

impl AiswebAirportJSON {
    pub fn new(path: &path::PathBuf) -> Self {
        let rdr = fs::File::open(path).unwrap();

        let deserialized_data: AiswebAirportJSON = serde_json::from_reader(rdr).unwrap();

        deserialized_data
    }

    pub fn write_pma_txt(&self, destination_path: &PathBuf) -> std::io::Result<()> {
        let mut buffer = fs::File::create(destination_path)?;

        for airport in self.features.iter() {
            let formated = format!(
                "Aeródromos_{}_{}_{}_Aeródromo_{}_{}_{}\n",
                airport.properties.opr,
                airport.properties.nome.trim(),
                airport.properties.localidade_id,
                airport.properties.latitude_dec,
                airport.properties.longitude_dec,
                (airport.properties.elevacao * 3.28084) as i32,
            );

            let (cow, _, _) = encoding_rs::WINDOWS_1252.encode(&formated);

            buffer.write(&cow);
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct AiswebVORJSON {
    features: Vec<Vor>,
}

#[derive(Deserialize, Debug)]
struct Vor {
    id: String,
    properties: VorProperties,
}

#[derive(Deserialize, Debug)]
struct VorProperties {
    ident: String,
    txt_name: String,
    latitude: f64,
    longitude: f64,
    frequency: f64,
    vortype: VorType,
}

#[derive(Deserialize, Debug)]
enum VorType {
    DVOR,
    VOR
}

impl AiswebVORJSON {
    pub fn new(path: &path::PathBuf) -> Self {
        let rdr = fs::File::open(path).unwrap();

        let deserialized_data: AiswebVORJSON = serde_json::from_reader(rdr).unwrap();

        deserialized_data
    }

    pub fn write_pma_txt(&self, destination_path: &PathBuf) -> std::io::Result<()> {
        let mut buffer = fs::File::create(destination_path)?;

        for vor in self.features.iter() {

            let vortype = match &vor.properties.vortype {
                VorType::DVOR => "DVOR",
                VorType::VOR => "VOR",
            };

            let vorsymbol = match &vor.properties.vortype {
                VorType::DVOR => "Navaid VOR-DME",
                VorType::VOR => "Navaid VOR-DME",
            };

            let formated = format!(
                "NAVAIDS_{}_{}_{}_{}_{}_{}_0\n",
                vortype,
                vor.properties.txt_name,
                vor.properties.ident,
                vorsymbol,
                vor.properties.latitude,
                vor.properties.longitude,
            );

            let (cow, _, _) = encoding_rs::WINDOWS_1252.encode(&formated);

            buffer.write(&cow);
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub enum Format {
    AiswebAirportJSON,
    AiswebVORJSON,
    AiswebFixesJSON
}

#[derive(Deserialize, Debug)]
struct AiswebFixesJSON {}