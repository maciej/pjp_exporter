use crate::metrics::{APIErrorLabels, APILatencyLabels, APIMetrics};
use crate::pjp::Param::{PM10, PM25};
use chrono::{DateTime, Utc};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use std::fmt::Debug;
use std::str::FromStr;
use std::time::Instant;

#[derive(Debug)]
pub(crate) enum Param {
    PM25,
    PM10,
}

impl FromStr for Param {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "PM2.5" {
            Ok(PM25)
        } else if s == "PM10" {
            Ok(PM10)
        } else {
            Err(())
        }
    }
}

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
    pub(crate) id: u32,
    #[serde(rename = "stationId")]
    pub(crate) station_id: u32,
    pub(crate) param: SensorParam,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SensorParam {
    #[serde(rename = "paramName")]
    pub(crate) name: String,
    #[serde(rename = "paramFormula")]
    pub(crate) formula: String,
    #[serde(rename = "paramCode")]
    pub(crate) code: String,
    #[serde(rename = "idParam")]
    pub(crate) id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetStationSensorsResp(Vec<Sensor>);

impl std::ops::Deref for GetStationSensorsResp {
    type Target = Vec<Sensor>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetDataResp {
    key: String,
    values: Vec<DataValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DataValue {
    // custom formats are described here https://serde.rs/custom-date-format.html
    #[serde(with = "data_timestamp_format")]
    date: DateTime<Utc>,
    value: Option<f64>,
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

#[derive(Clone)]
pub(crate) struct API {
    metrics: APIMetrics,
}

impl API {
    pub(crate) fn new(api_metrics: APIMetrics) -> Self {
        API {
            metrics: api_metrics,
        }
    }

    pub(crate) async fn find_all_stations(&self) -> reqwest::Result<FindAllStationsResp> {
        let endpoint = "/pjp-api/rest/station/findAll";
        let start = Instant::now();

        let response = reqwest::get("https://api.gios.gov.pl/pjp-api/rest/station/findAll").await?;
        let response = self.on_response(response, endpoint)?;

        let result = response.json::<FindAllStationsResp>().await;

        let latency = start.elapsed();
        self.metrics
            .latency_seconds
            .get_or_create(&APILatencyLabels {
                endpoint: endpoint.to_owned(),
            })
            .observe(latency.as_secs_f64());

        result
    }

    pub(crate) async fn get_station_sensors(
        &self,
        station_id: u32,
    ) -> reqwest::Result<GetStationSensorsResp> {
        let endpoint = "/pjp-api/rest/station/sensors/{station_id}";
        let start = Instant::now();

        let response = reqwest::get(format!(
            "https://api.gios.gov.pl/pjp-api/rest/station/sensors/{station_id}",
            station_id = station_id
        ))
        .await?;

        let response = self.on_response(response, endpoint)?;

        let result = response.json::<GetStationSensorsResp>().await;

        let latency = start.elapsed();
        self.metrics
            .latency_seconds
            .get_or_create(&APILatencyLabels {
                endpoint: endpoint.to_owned(),
            })
            .observe(latency.as_secs_f64());

        result
    }

    fn on_response(&self, res: Response, endpoint: &str) -> reqwest::Result<Response> {
        match res.error_for_status() {
            Ok(_res) => Ok(_res),
            Err(err) => {
                let status_code = err.status().unwrap().as_u16();

                let labels = &APIErrorLabels {
                    endpoint: endpoint.to_owned(),
                    code: status_code,
                };
                self.metrics.errors.get_or_create(labels).inc();

                Err(err)
            }
        }
    }

    pub(crate) async fn get_data(&self, sensor_id: u32) -> reqwest::Result<GetDataResp> {
        let endpoint = "/pjp-api/rest/data/getData/{sensor_id}";
        let start = Instant::now();

        let response = reqwest::get(format!(
            "https://api.gios.gov.pl/pjp-api/rest/data/getData/{sensor_id}",
            sensor_id = sensor_id
        ))
        .await?;

        let response = self.on_response(response, endpoint)?;

        let result = response.json::<GetDataResp>().await;

        let latency = start.elapsed();
        self.metrics
            .latency_seconds
            .get_or_create(&APILatencyLabels {
                endpoint: endpoint.to_owned(),
            })
            .observe(latency.as_secs_f64());

        result
    }

    pub(crate) async fn get_latest_data(&self, sensor_id: u32) -> reqwest::Result<DataValue> {
        // TODO next
        todo!()
    }
}
