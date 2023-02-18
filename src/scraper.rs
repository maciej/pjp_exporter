use crate::metrics::AirQualityMetrics;
use crate::pjp;

pub enum ScrapeError {
    HTTPError(reqwest::Error),
}

pub(crate) struct Scraper {
    metrics: AirQualityMetrics,
    api: pjp::API,
}

impl Scraper {
    pub(crate) fn new(api: pjp::API, metrics: AirQualityMetrics) -> Scraper {
        Scraper { api, metrics }
    }

    pub(crate) fn scrape_station(&self) -> Result<(), ScrapeError> {
        Ok(())
    }
}
