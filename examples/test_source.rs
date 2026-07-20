use babel_core::flaresolverr::download_and_save_source;

#[tokio::main]
async fn main() {
    let flaresolverr_url = "http://localhost:8191/v1";
    let target_url = "https://example.com";

    match download_and_save_source(flaresolverr_url, target_url).await {
        Ok(path) => {
            println!("Success: {:?}", path);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
