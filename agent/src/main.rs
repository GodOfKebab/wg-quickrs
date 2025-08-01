use clap::Parser;
use log::LevelFilter;
use once_cell::sync::OnceCell;
use simple_logger::SimpleLogger;
use std::path::PathBuf;

mod api;
mod app;
mod conf;
mod macros;
mod server;
mod wireguard;

#[derive(Parser, Debug)]
#[command(
    version = full_version!(),
    about = "Run the wg-rusteze network agent",
    long_about = "Starts the wg-rusteze agent with the default/provided configuration file. \
                  Use this tool to manage the peer and network configuration of the \
                  WireGuard-based overlay network over the web console."
)]
struct Args {
    #[arg(
        long,
        default_value = ".wg-rusteze/conf.yml",
        value_name = "WG_RUSTEZE_CONFIG_FILE_PATH"
    )]
    wg_rusteze_config_file: PathBuf,
    #[arg(
        long,
        default_value = "/opt/homebrew/etc/wireguard/",
        value_name = "WIREGUARD_CONFIG_FOLDER_PATH"
    )]
    wireguard_config_folder: PathBuf,
    #[arg(
        long,
        default_value = ".wg-rusteze/cert.pem",
        value_name = "TLS_CERTIFICATE_FILE_PATH"
    )]
    tls_cert: PathBuf,
    #[arg(
        long,
        default_value = ".wg-rusteze/key.pem",
        value_name = "TLS_PRIVATE_KEY_FILE_PATH"
    )]
    tls_key: PathBuf,
}

pub static WG_RUSTEZE_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();
pub static WIREGUARD_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // print version and start logger
    println!(full_version!());
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap_or_else(|e| {
            eprintln!("Logger init failed: {e}");
        });

    // get the wg_rusteze config file path
    let args = Args::parse();
    WG_RUSTEZE_CONFIG_FILE
        .set(args.wg_rusteze_config_file)
        .expect("Failed to set WG_RUSTEZE_CONFIG_FILE");
    log::info!(
        "using the wg-rusteze config file at \"{}\"",
        WG_RUSTEZE_CONFIG_FILE.get().unwrap().display()
    );

    // get the wireguard config file path
    let config = conf::util::get_config();
    let wireguard_config_filename = format!("{}.conf", config.network.identifier);
    let wireguard_config_file_path = args.wireguard_config_folder.join(wireguard_config_filename);
    WIREGUARD_CONFIG_FILE
        .set(wireguard_config_file_path)
        .expect("Failed to set WIREGUARD_CONFIG_FILE");
    log::info!(
        "using the wireguard config file at \"{}\"",
        WIREGUARD_CONFIG_FILE.get().unwrap().display()
    );

    // start the tunnel
    wireguard::cmd::start_tunnel(&config).unwrap_or_else(|e| {
        log::error!("{e}");
    });

    // start the HTTP server with TLS for frontend and API control
    server::run_http_server(&config, &args.tls_cert, &args.tls_key).await
}
