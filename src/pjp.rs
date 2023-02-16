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


#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Sensor {
    id: u32,
    #[serde(rename = "stationId")]
    station_id: u32,
    param: SensorParam,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SensorParam {
    #[serde(rename = "paramName")]
    name: String,
    #[serde(rename = "paramFormula")]
    formula: String,
    #[serde(rename = "paramCode")]
    code: String,
    #[serde(rename = "idParam")]
    id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetStationSensorsResp(Vec<Sensor>);

pub(crate) async fn find_all_stations() -> reqwest::Result<FindAllStationsResp> {
    reqwest::get("https://api.gios.gov.pl/pjp-api/rest/station/findAll")
        .await?
        .json::<FindAllStationsResp>()
        .await
}

pub(crate) async fn get_station_sensors(station_id: u32) -> reqwest::Result<GetStationSensorsResp> {
    reqwest::get(format!("https://api.gios.gov.pl/pjp-api/rest/station/sensors/{station_id}", station_id = station_id))
        .await?
        .json::<GetStationSensorsResp>()
        .await
}
