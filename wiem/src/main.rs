mod app;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let app = app::App::new()?;
    app.run().await?;

    Ok(())
}
