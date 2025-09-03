use server_enclave::run_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "5001".to_string())
        .parse::<u16>()?;

    println!("Starting Pass Wallet server on port {}", port);
    run_server(port).await?;

    Ok(())
}
