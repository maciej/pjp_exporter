use crate::metrics::APIMetrics;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use std::fmt::Debug;
use std::time::Instant;

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
    #[serde(
        rename = "gegrLat",
        deserialize_with = "deserialize_number_from_string"
    )]
    lat: f64,
    #[serde(
        rename = "gegrLon",
        deserialize_with = "deserialize_number_from_string"
    )]
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

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetDataResp {
    key: String,
    values: Vec<DataValues>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DataValues {
    // custom formats are described here https://serde.rs/custom-date-format.html
    #[serde(with = "data_timestamp_format")]
    date: DateTime<Utc>,
    value: f64,
}

mod data_timestamp_format {
    use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
    use chrono_tz::Europe::Warsaw;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let ndt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;

        let ldt = Warsaw.from_local_datetime(&ndt).unwrap();

        Ok(ldt.with_timezone(&Utc))
    }
}

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
        self.api_metrics
            .latency_seconds
            .observe(latency.as_secs_f64());

        resp
    }

    pub(crate) async fn get_station_sensors(
        &self,
        station_id: u32,
    ) -> reqwest::Result<GetStationSensorsResp> {
        let start = Instant::now();
        let resp = reqwest::get(format!(
            "https://api.gios.gov.pl/pjp-api/rest/station/sensors/{station_id}",
            station_id = station_id
        ))
        .await?
        .json::<GetStationSensorsResp>()
        .await;

        let latency = start.elapsed();
        self.api_metrics
            .latency_seconds
            .observe(latency.as_secs_f64());

        resp
    }

    pub(crate) async fn get_data(&self, sensor_id: u32) -> reqwest::Result<GetDataResp> {
        let start = Instant::now();
        let resp = reqwest::get(format!(
            "https://api.gios.gov.pl/pjp-api/rest/data/getData/{sensor_id}",
            sensor_id = sensor_id
        ))
        .await?
        .json::<GetDataResp>()
        .await;

        let latency = start.elapsed();
        self.api_metrics
            .latency_seconds
            .observe(latency.as_secs_f64());

        resp
    }
}
