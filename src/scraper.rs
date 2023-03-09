use crate::metrics::{AirQualityLabels, AirQualityMetrics};
use crate::pjp;
use crate::pjp::{GetStationSensorsResp, Param};
use crate::scraper::ScrapeError::{HTTPError, NoDataError, NoDataValueError};
use std::str::FromStr;
use std::sync::Arc;

pub enum ScrapeError {
    HTTPError(reqwest::Error),
    TokioTaskError(tokio::task::JoinError),
    NoDataError,
    NoDataValueError,
}

impl From<reqwest::Error> for ScrapeError {
    fn from(value: reqwest::Error) -> Self {
        HTTPError(value)
    }
}

impl From<tokio::task::JoinError> for ScrapeError {
    fn from(value: tokio::task::JoinError) -> Self {
        ScrapeError::TokioTaskError(value)
    }
}

/// Scrapes pjp::API and reports Air Quality Metrics
pub(crate) struct Scraper {
    metrics: AirQualityMetrics,
    api: Arc<pjp::API>,
}

#[derive(Debug)]
struct Reading {
    station_id: u32,
    sensor_id: u32,
    param: Param,
    val: f64,
}

impl Scraper {
    pub(crate) fn new(api: pjp::API, metrics: AirQualityMetrics) -> Scraper {
        Scraper {
            api: Arc::new(api),
            metrics,
        }
    }

    pub(crate) async fn scrape_station(&self, station_id: u32) -> Result<(), ScrapeError> {
        // First get station sensors
        let sensors: GetStationSensorsResp = self.api.get_station_sensors(station_id).await?;

        let known_sensors: Vec<(Param, u32)> = sensors
            .iter()
            .filter(|s| pjp::Param::from_str(&s.param.code).is_ok())
            .map(|s| (pjp::Param::from_str(&s.param.code).unwrap(), s.id))
            .collect();

        let mut set = tokio::task::JoinSet::new();

        for s in known_sensors {
            let api = self.api.clone();
            set.spawn(async move {
                let r: Result<f64, ScrapeError> = api
                    .get_latest_data(s.1)
                    .await
                    .map_err(|err| HTTPError(err))
                    .and_then(|data_opt| data_opt.ok_or_else(|| NoDataError))
                    .and_then(|data| data.value.ok_or_else(|| NoDataValueError));
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
            };
            match reading.param {
                Param::PM10 => self.metrics.pm10.get_or_create(&labels).set(reading.val),
                Param::PM25 => self.metrics.pm25.get_or_create(&labels).set(reading.val),
            };
        }

        Ok(())
    }
}
