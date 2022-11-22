use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, path};

#[derive(Deserialize, Debug)]
pub struct AiswebJSON {
    features: Vec<Object>,
}

#[derive(Deserialize, Debug)]
struct Object {
    properties: ObjectProperties,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ObjectProperties {
    Airport {
        localidade_id: String,
        nome: String,
        opr: String,
        latitude_dec: f64,
        longitude_dec: f64,
        elevacao: f64,
        airport_pk: Option<u16>,
    },
    Vor {
        ident: String,
        txtname: String,
        latitude: f64,
        longitude: f64,
        frequency: f64,
        vortype: VorType,
    },
    Fixes {
        ident: String,
        latitude: f64,
        longitude: f64,
        codetype: String,
    },
    Thresholds {
        rwyendid: String,
        threshlat: f64,
        threshlon: f64,
        threshelev: Option<f64>,
        gid: u32,
        runway_pk: u16,
    },
    RunwayV2 {
        runway_pk: u16,
        airport_pk: u16,
        surface: String,
    },
    NDB {
        codeid: String,
        geolat: f64,
        geolong: f64,
        txtname: String,
        valfreq: f64,
        tipo: String,
    },
}

impl AiswebJSON {
    pub fn new(path: &path::PathBuf) -> Self {
        let rdr = fs::File::open(path).unwrap();

        let deserialized_data: AiswebJSON = serde_json::from_reader(rdr).unwrap();

        deserialized_data
    }

    pub fn compare(struct1: AiswebJSON, struct2: AiswebJSON) {
        todo!()
    }

    pub fn decide(&self, destination_path: &PathBuf) {
        let mut buffer = fs::File::create(destination_path).unwrap();
        for object in self.features.iter() {
            match &object.properties {
                ObjectProperties::Airport {
                    localidade_id,
                    nome,
                    opr,
                    latitude_dec,
                    longitude_dec,
                    elevacao,
                    airport_pk,
                } => {
                    let formated = format!(
                        "Aeródromos_{}_{}_{}_Aeródromo_{}_{}_{}\n",
                        opr,
                        nome.trim(),
                        localidade_id,
                        latitude_dec,
                        longitude_dec,
                        (elevacao * 3.28084) as i32,
                    );

                    write_pma_txt(&mut buffer, &formated).unwrap();
                }
                ObjectProperties::Vor {
                    ident,
                    txtname,
                    latitude,
                    longitude,
                    frequency,
                    vortype,
                } => {
                    let vortype_str = match &vortype {
                        VorType::DVOR => "DVOR",
                        VorType::VOR => "VOR",
                    };

                    let formated = format!(
                        "{}_{:.2}_{}_{}_Padrão_{}_{}_0\n",
                        vortype_str, frequency, txtname, ident, latitude, longitude,
                    );

                    write_pma_txt(&mut buffer, &formated).unwrap();
                }
                ObjectProperties::Fixes {
                    ident,
                    latitude,
                    longitude,
                    codetype,
                } => {
                    let formated = format!(
                        "AISWEB_{}_ _{}_Padrão_{}_{}_0\n",
                        codetype.replace('_', "-"),
                        ident,
                        latitude,
                        longitude
                    );
                    write_pma_txt(&mut buffer, &formated).unwrap();
                }
                ObjectProperties::Thresholds {
                    rwyendid,
                    threshlat,
                    threshlon,
                    threshelev,
                    gid,
                    runway_pk,
                } => {
                    let elevation = threshelev.unwrap_or(0.0);

                    if threshlat.to_owned() == 0.0 && threshlon.to_owned() == 0.0 {
                        continue;
                    }

                    let formated = format!(
                        "Cabeceiras_ _ _{} - {}_Padrão_{}_{}_{}\n",
                        rwyendid,
                        gid,
                        threshlat,
                        threshlon,
                        (elevation * 3.28084) as i32
                    );

                    write_pma_txt(&mut buffer, &formated).unwrap();
                }
                ObjectProperties::RunwayV2 {
                    runway_pk,
                    airport_pk,
                    surface,
                } => break,
                ObjectProperties::NDB {
                    codeid,
                    geolat,
                    geolong,
                    txtname,
                    valfreq,
                    tipo,
                } => {
                    let formated = format!(
                        "{}_{}_{}_{}_Padrão_{}_{}_0\n",
                        tipo, valfreq, txtname, codeid, geolat, geolong,
                    );

                    write_pma_txt(&mut buffer, &formated).unwrap();
                },
            }
        }
    }
}

#[derive(Deserialize, Debug)]
enum VorType {
    DVOR,
    VOR,
}

#[derive(Deserialize, Debug)]
struct AiswebFixesJSON {}

pub fn write_pma_txt(buffer: &mut fs::File, formated: &str) -> std::io::Result<()> {
    let (cow, _, _) = encoding_rs::WINDOWS_1252.encode(&formated);

    match buffer.write(&cow) {
        Ok(_) => return Ok(()),
        Err(e) => {
            eprint!("Somenthing wrong happened: {}", e);
        }
    }
    Ok(())
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
