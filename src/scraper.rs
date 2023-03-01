use crate::metrics::AirQualityMetrics;
use crate::pjp;
use crate::pjp::GetStationSensorsResp;
use std::str::FromStr;
use std::sync::Arc;

pub enum ScrapeError {
    HTTPError(reqwest::Error),
    TokioTaskError(tokio::task::JoinError),
}

impl From<reqwest::Error> for ScrapeError {
    fn from(value: reqwest::Error) -> Self {
        ScrapeError::HTTPError(value)
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

        let known_sensors: Vec<(pjp::Param, u32)> = sensors
            .iter()
            .filter(|s| pjp::Param::from_str(&s.param.code).is_ok())
            .map(|s| (pjp::Param::from_str(&s.param.code).unwrap(), s.id))
            .collect();

        let mut set = tokio::task::JoinSet::new();

        for s in known_sensors {
            let api = self.api.clone();
            set.spawn(async move { (s.0, api.get_latest_data(s.1).await) });
        }

        while let Some(r) = set.join_next().await {
            let out = r?;
            let data = out.1?;
            println!("{:#?}, {:#?}", out.0, data);
        }

        Ok(())
    }
}
