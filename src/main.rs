mod pjp;

const PJP_STATION_FIND_ALL_URI: &str = "https://api.gios.gov.pl/pjp-api/rest/station/findAll";


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get(PJP_STATION_FIND_ALL_URI)
        .await?
        .json::<pjp::FindAllStationsResp>()
        .await?;
    println!("{:#?}", resp);
    Ok(())
}
