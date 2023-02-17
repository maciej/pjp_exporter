mod pjp;
mod metrics;

use prometheus_client::registry::Registry;
use crate::pjp::API;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Registry::default();
    let metrics = metrics::Metrics::new(&mut registry);

    let pjp_api = API::new(metrics.api_metrics);

    let resp = pjp_api.find_all_stations().await?;
    println!("{:#?}", resp);

    let resp = pjp_api.get_station_sensors(530).await?;
    println!("{:#?}", resp);

    let resp = pjp_api.get_data(3585).await?;
    println!("{:#?}", resp);

    Ok(())
}
