use crate::metrics::AirQualityMetrics;
use crate::pjp;
use crate::pjp::GetStationSensorsResp;
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

        let _known_sensors: Vec<u32> = sensors
            .iter()
            .filter(|s| s.param.code == "PM2.5" || s.param.code == "PM10")
            .map(|s| s.id)
            .collect();

        let mut set = tokio::task::JoinSet::new();

        for s in _known_sensors {
            let api = self.api.clone();
            set.spawn(async move { api.get_data(s).await });
        }

        while let Some(r) = set.join_next().await {
            let out = r??;
            println!("{:#?}", out);
        }

        Ok(())
    }
}
