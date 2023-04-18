use std::error::Error;
use tokio::runtime::Runtime;

mod client;
mod server;

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    server::start();

    let mut runtime = Runtime::new().unwrap();

    let task = || async {
        let client = client::Client::new()?;

        let req = client
            .request(reqwest::Method::GET, "http://localhost:1729")
            .build()?;

        let text = client.execute(req).await?.text().await?;
        log::info!("Request successful: {}", &text[..]);

        let res: Result<_, Box<dyn Error>> = Ok(());
        res
    };

    let join_handle = runtime.spawn(async move {
        match task().await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Something went wrong: {}", e);
            }
        }
    });

    runtime.block_on(join_handle).unwrap();

    Ok(())
}
