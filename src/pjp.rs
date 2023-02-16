use serde_aux::field_attributes::deserialize_number_from_string;
use std::fmt::Debug;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use crate::metrics::APIMetrics;


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

pub(crate) struct API {
    api_metrics: APIMetrics,
}

impl API {
    pub(crate) fn new(api_metrics: APIMetrics) -> Self {
        API { api_metrics }
    }

    pub(crate) async fn find_all_stations(&self) -> reqwest::Result<FindAllStationsResp> {
        let start = Instant::now();
        let resp = reqwest::get("https://api.gios.gov.pl/pjp-api/rest/station/findAll")
            .await?
            .json::<FindAllStationsResp>()
            .await;
        let latency = start.elapsed();
        self.api_metrics.latency_seconds.observe(latency.as_secs_f64());

        resp
    }

    pub(crate) async fn get_station_sensors(&self, station_id: u32) -> reqwest::Result<GetStationSensorsResp> {
        let start = Instant::now();
        let resp = reqwest::get(format!("https://api.gios.gov.pl/pjp-api/rest/station/sensors/{station_id}", station_id = station_id))
            .await?
            .json::<GetStationSensorsResp>()
            .await;

        let latency = start.elapsed();
        self.api_metrics.latency_seconds.observe(latency.as_secs_f64());

        resp
    }
}


