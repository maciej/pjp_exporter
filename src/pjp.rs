use serde_aux::field_attributes::deserialize_number_from_string;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Commune {
    #[serde(rename = "communeName")]
    commune: String,
    #[serde(rename = "districtName")]
    district: String,
    #[serde(rename = "provinceName")]
    province: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct City {
    commune: Commune,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Station {
    id: u32,
    #[serde(rename = "stationName")]
    name: String,
    #[serde(rename = "gegrLat", deserialize_with = "deserialize_number_from_string")]
    lat: f64,
    #[serde(rename = "gegrLon", deserialize_with = "deserialize_number_from_string")]
    lon: f64,
    city: City,
    #[serde(rename = "addressStreet")]
    street_address: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FindAllStationsResp(Vec<Station>);
