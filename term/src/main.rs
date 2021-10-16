mod app;

use url::Url;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let default_url = format!("ws://localhost:{}", 8080);
    let matches = clap::App::new("ebterm")
        .bin_name(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::with_name("server-url")
                .long("url")
                .help("URL of the server")
                .default_value(&default_url)
                .validator(|s| match Url::parse(&s) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err.to_string()),
                }),
        )
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    let level = match matches.occurrences_of("verbose") {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        2 => tracing::Level::TRACE,
        n => panic!("cannot accept {} occurences of -v flag", n),
    };
    tracing_subscriber::fmt()
        .with_writer(|| {
            let base_directories = xdg::BaseDirectories::with_prefix(clap::crate_name!()).unwrap();
            let log_file_path = base_directories
                .get_cache_home()
                .join(format!("{}.log", clap::crate_name!()));
            if !log_file_path.exists() {
                std::fs::create_dir_all(&log_file_path.parent().unwrap()).unwrap();
            }
            let log_file = std::fs::OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(&log_file_path)
                .unwrap();
            log_file
        })
        .with_max_level(level)
        .init();

    let server_url = Url::parse(matches.value_of("server-url").unwrap()).unwrap();
    let client = eb_rpc::connect_ws(&server_url).await?;

    let app = app::App::new(client)?;
    app.run().await?;

    Ok(())
}
