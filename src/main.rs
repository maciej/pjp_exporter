mod pjp;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = pjp::find_all_stations().await?;
    println!("{:#?}", resp);

    let resp = pjp::get_station_sensors(530).await?;
    println!("{:#?}", resp);

    Ok(())
}
