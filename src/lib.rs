use serde::Deserialize;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

trait Format {
    fn fmt(&self) -> String;
}

pub trait WritingDB {
    fn write_pma_txt(&self, option: Options, pathbuf: &PathBuf) -> std::io::Result<()>;
}

#[derive(Debug)]
pub enum Options {
    Airport,
    VOR,
    NDB,
    Fixes,
    Thresholds,
    RunwayV2,
    CompleteThresholds,
}

impl Display for Options {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            Options::Airport => f.write_str("airport"),
            Options::Fixes => f.write_str("waypoint"),
            Options::NDB => f.write_str("ndb"),
            Options::RunwayV2 => f.write_str("runway_v2"),
            Options::Thresholds => f.write_str("rwydirection"),
            Options::VOR => f.write_str("vor"),
            Options::CompleteThresholds => f.write_str("cabeceiras"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct AiswebJSON<T> {
    pub features: Vec<Object<T>>,
}

impl<T> WritingDB for AiswebJSON<T>
where
    T: Format,
{
    fn write_pma_txt(&self, opt: Options, path: &PathBuf) -> std::io::Result<()> {
        let filename = PathBuf::from(format!("./aisweb_{}", opt)).with_extension("txt");

        let final_dest = treat_path(path, &filename);

        let mut file = match std::fs::File::create(final_dest) {
            Err(e) => {
                eprintln!("Something wrong happened: {e}");
                File::create(filename)?
            }
            Ok(f) => f,
        };

        for feature in self.features.iter() {
            let formatted = feature.properties.fmt();
            let (cow, _, _) = encoding_rs::WINDOWS_1252.encode(&formatted);

            match file.write(&cow) {
                Ok(_) => {}
                Err(e) => {
                    eprint!("Somenthing wrong happened: {}", e);
                }
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct Object<T> {
    pub properties: T,
}

#[derive(Deserialize, Debug)]
pub struct Airport {
    localidade_id: String,
    nome: String,
    opr: String,
    latitude_dec: f64,
    longitude_dec: f64,
    elevacao: f64,
    pub airport_pk: Option<u16>,
}

impl Format for Airport {
    fn fmt(&self) -> String {
        let formatted = format!(
            "Aeródromos_{}_{}_{}_Aeródromo_{}_{}_{}\n",
            self.opr,
            self.nome.trim(),
            self.localidade_id,
            self.latitude_dec,
            self.longitude_dec,
            (self.elevacao * 3.28084) as i32,
        );
        formatted
    }
}

#[derive(Deserialize, Debug)]
pub struct Vor {
    ident: String,
    txtname: String,
    latitude: f64,
    longitude: f64,
    frequency: f64,
    vortype: VorType,
}

impl Format for Vor {
    fn fmt(&self) -> String {
        let formatted = format!(
            "{:?}_{:.2}_{}_{}_Padrão_{}_{}_0\n",
            self.vortype, self.frequency, self.txtname, self.ident, self.latitude, self.longitude,
        );
        formatted
    }
}

#[derive(Deserialize, Debug)]
pub struct Fixes {
    ident: String,
    latitude: f64,
    longitude: f64,
    codetype: String,
}

impl Format for Fixes {
    fn fmt(&self) -> String {
        let formatted = format!(
            "Fixos_{}_ _{}_Padrão_{}_{}_0\n",
            self.codetype.replace('_', "-"),
            self.ident,
            self.latitude,
            self.longitude
        );
        formatted
    }
}

#[derive(Deserialize, Debug)]
pub struct Thresholds {
    rwyendid: String,
    threshlat: f64,
    threshlon: f64,
    threshelev: Option<f64>,
    pub runway_pk: u16,
}

#[derive(Deserialize, Debug)]
pub struct RunwayV2 {
    pub runway_pk: u16,
    pub airport_pk: u16,
    surface: String,
    runwayleng: f64,
    width: f64,
}

#[derive(Deserialize, Debug)]
pub struct NDB {
    codeid: String,
    geolat: f64,
    geolong: f64,
    txtname: String,
    valfreq: f64,
    tipo: String,
}

impl Format for NDB {
    fn fmt(&self) -> String {
        format!(
            "{}_{}_{}_{}_Padrão_{}_{}_0\n",
            self.tipo, self.txtname, self.valfreq, self.codeid, self.geolat, self.geolong
        )
    }
}

#[derive(Debug)]
pub struct CompleteThresholds {
    localidade_id: String,
    rwyendid: String,
    threshlat: f64,
    threshlon: f64,
    threshelev: Option<f64>,
    surface: String,
    runwayleng: f64,
    width: f64,
}

impl Format for CompleteThresholds {
    fn fmt(&self) -> String {
        let elevation = match self.threshelev {
            None => 0,
            Some(x) => (x * 3.28084) as i32,
        };
        format!(
            "{}_{}_{}x{}_{}_Padrão_{}_{}_{}\n",
            self.localidade_id,
            self.surface,
            self.runwayleng,
            self.width,
            self.rwyendid,
            self.threshlat,
            self.threshlon,
            elevation
        )
    }
}

impl CompleteThresholds {
    pub fn new(airport: &Airport, runway: &RunwayV2, threshold: &Thresholds) -> Self {
        let len = airport.localidade_id.len();
        let slice = &airport.localidade_id[len - 2..];

        CompleteThresholds {
            localidade_id: airport.localidade_id.to_owned(),
            rwyendid: format!("{}{}", slice, threshold.rwyendid),
            threshlat: threshold.threshlat,
            threshlon: threshold.threshlon,
            threshelev: threshold.threshelev,
            surface: runway.surface.to_owned(),
            runwayleng: runway.runwayleng,
            width: runway.width,
        }
    }
}

impl WritingDB for Vec<CompleteThresholds> {
    fn write_pma_txt(&self, opt: Options, path: &PathBuf) -> std::io::Result<()> {
        let filename = PathBuf::from(format!("./aisweb_{}", opt)).with_extension("txt");

        let final_dest = treat_path(path, &filename);

        let mut file = match std::fs::File::create(final_dest) {
            Err(e) => {
                eprintln!("Something wrong happened: {e}");
                File::create(filename)?
            }
            Ok(f) => f,
        };

        for thresh in self.iter() {
            let formatted = thresh.fmt();
            let (cow, _, _) = encoding_rs::WINDOWS_1252.encode(&formatted);

            match file.write(&cow) {
                Ok(_) => {}
                Err(e) => {
                    eprint!("Somenthing wrong happened: {}", e);
                }
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
enum VorType {
    DVOR,
    VOR,
}

fn treat_path(directory: &PathBuf, filename: &PathBuf) -> PathBuf {
    if directory.is_dir() {
        directory.join(filename)
    } else {
        filename.to_owned()
    }
}

mod haversine {
    const R: f64 = 6371e+3; // metres

    struct LatsLongs {
        lat1: f64,
        lat2: f64,
        long1: f64,
        long2: f64,
    }

    impl LatsLongs {
        fn distance(&self) -> f64 {
            let φ1 = self.lat1.to_radians();
            let φ2 = self.lat2.to_radians();
            let Δφ = φ2 - φ1;
            let Δλ = (self.long2 - self.long1).to_radians();

            let a = (Δφ / 2.0).sin().powi(2) + (φ1).cos() * (φ2).cos() * (Δλ / 2.0).sin().powi(2);

            let c = 2.0 * ((a).sqrt().atan2((1.0 - a).sqrt()));

            R * c
        }
    }
}
