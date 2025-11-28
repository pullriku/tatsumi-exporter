use std::env;

use dotenvy::dotenv;
use tatsumi_exporter::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let host =
        env::var("EXPORTER_HOST").unwrap_or_else(|_e| "0.0.0.0".to_string());
    let port: u16 = env::var("EXPORTER_PORT")
        .unwrap_or_else(|_e| "14000".to_string())
        .parse()
        .expect("Failed to parse port.");
    server::serve((host, port)).await
}
