use crate::metrics::{AirQualityLabels, AirQualityMetrics};
use crate::pjp;
use crate::pjp::{GetStationSensorsResp, Param};
use crate::scraper::ScrapeError::{Http, NoData, NoDataValue};
use std::str::FromStr;
use std::sync::Arc;

pub enum ScrapeError {
    Http(reqwest::Error),
    TokioTask(tokio::task::JoinError),
    NoData,
    NoDataValue,
}

impl From<reqwest::Error> for ScrapeError {
    fn from(value: reqwest::Error) -> Self {
        Http(value)
    }
}

impl From<tokio::task::JoinError> for ScrapeError {
    fn from(value: tokio::task::JoinError) -> Self {
        ScrapeError::TokioTask(value)
    }
}

/// Scrapes pjp::API and reports Air Quality Metrics
pub(crate) struct Scraper {
    metrics: AirQualityMetrics,
    api: Arc<pjp::Api>,
}

#[derive(Debug)]
struct Reading {
    station_id: u32,
    sensor_id: u32,
    param: Param,
    val: f64,
}

impl Scraper {
    pub(crate) fn new(api: pjp::Api, metrics: AirQualityMetrics) -> Scraper {
        Scraper {
            api: Arc::new(api),
            metrics,
        }
    }

    pub(crate) async fn scrape_station(
        &self,
        station_id: u32,
        name: &str,
    ) -> Result<(), ScrapeError> {
        // First get station sensors
        let sensors: GetStationSensorsResp = self.api.get_station_sensors(station_id).await?;

        let known_sensors: Vec<(Param, u32)> = sensors
            .iter()
            .filter(|s| Param::from_str(&s.param.code).is_ok())
            .map(|s| (Param::from_str(&s.param.code).unwrap(), s.id))
            .collect();

        let mut set = tokio::task::JoinSet::new();

        for s in known_sensors {
            let api = self.api.clone();
            set.spawn(async move {
                let r: Result<f64, ScrapeError> = api
                    .get_latest_data(s.1)
                    .await
                    .map_err(Http)
                    .and_then(|data_opt| data_opt.ok_or_else(|| NoData))
                    .and_then(|data| data.value.ok_or_else(|| NoDataValue));
                r.map(|data| Reading {
                    sensor_id: s.1,
                    station_id,
                    param: s.0,
                    val: data,
                })
            });
        }

        while let Some(r) = set.join_next().await {
            let reading = r??;

            // println!("{:#?}", &reading);
            let labels = AirQualityLabels {
                station: reading.station_id,
                sensor: reading.sensor_id,
                station_name: String::from(name),
            };
            match reading.param {
                Param::PM10 => self.metrics.pm10.get_or_create(&labels).set(reading.val),
                Param::PM25 => self.metrics.pm25.get_or_create(&labels).set(reading.val),
            };
        }

        Ok(())
    }
}
