#[macro_use]
extern crate lazy_mut;
#[macro_use]
extern crate lazy_static;
extern crate serde_derive;
extern crate serde_yaml;

use std::time::Instant;
use std::{fs, path::Path};

use prometheus::{Encoder, IntGaugeVec, Opts, Registry, TextEncoder};
use warp::Filter;

mod collectors;
mod config;
mod mastodon;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();

     // Ratelimit
     static ref MASTODON_RATELIMIT_REMAINING: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "mastodon_ratelimit_remaining",
            "Current remaining ratelimit of instance.",
        ),
        &["instance"],
    )
    .unwrap();

    // Ratelimit reset
    static ref MASTODON_RATELIMIT_RESET: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "mastodon_ratelimit_reset",
            "Number of seconds since 1970 of ratelimit reset for instance.",
        ),
        &["instance"],
    ).unwrap();

    // Info
    static ref MASTODON_INFO: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "mastodon_info",
            "General instance information.",
        ),
        &["instance", "domain", "title", "version"],
    )
    .unwrap();

    // Registration enabled
    static ref MASTODON_REGISTRATIONS_ENABLED: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "mastodon_registrations_enabled",
            "Whether or not registrations are enabled on instance.",
        ),
        &["instance"],
    )
    .unwrap();

    // Registration approval required
    static ref MASTODON_REGISTRATIONS_APPROVAL_REQUIRED: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "mastodon_registrations_approval_required",
            "Whether or not approval is required on instance.",
        ),
        &["instance"],
    )
    .unwrap();

    // Account followers count
    static ref MASTODON_ACCOUNT_FOLLOWERS_COUNT: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "mastodon_account_followers_count",
            "Number of followers for account.",
        ),
        &["instance", "account_id", "username"],
    ).unwrap();

    // Account following count
    static ref MASTODON_ACCOUNT_FOLLOWING_COUNT: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "mastodon_account_following_count",
            "Number of accounts followed by account.",
        ),
        &["instance", "account_id", "username"],
    ).unwrap();

    // Account statuses count
    static ref MASTODON_ACCOUNT_STATUSES_COUNT: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "mastodon_account_statuses_count",
            "Number of statuses for account.",
        ),
        &["instance", "account_id", "username"],
    ).unwrap();

    // Account last status at
    static ref MASTODON_ACCOUNT_LAST_STATUS_AT: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "mastodon_account_last_status_at",
            "Number of seconds since 1970 of last status for account.",
        ),
        &["instance", "account_id", "username"],
    ).unwrap();
}

lazy_mut! {
    static mut INSTANCES: Vec<String> = Vec::new();
    static mut ACCOUNTS: Vec<(String, String)> = Vec::new();
}

async fn metrics() -> Result<impl warp::Reply, warp::Rejection> {
    let start = Instant::now();

    println!("Collecting metrics...");

    collectors::instance::collect_instances().await.ok();
    collectors::account::collect_accounts().await.ok();

    println!("Collecting all metrics done in {:?}", start.elapsed());
    println!();

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    Ok(String::from_utf8(buffer).unwrap())
}

#[tokio::main]
async fn main() {
    REGISTRY
        .register(Box::new(MASTODON_RATELIMIT_REMAINING.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(MASTODON_RATELIMIT_RESET.clone()))
        .unwrap();
    REGISTRY.register(Box::new(MASTODON_INFO.clone())).unwrap();
    REGISTRY
        .register(Box::new(MASTODON_REGISTRATIONS_ENABLED.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(MASTODON_REGISTRATIONS_APPROVAL_REQUIRED.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(MASTODON_ACCOUNT_FOLLOWERS_COUNT.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(MASTODON_ACCOUNT_FOLLOWING_COUNT.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(MASTODON_ACCOUNT_STATUSES_COUNT.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(MASTODON_ACCOUNT_LAST_STATUS_AT.clone()))
        .unwrap();

    let config_file_name = "mastodon_exporter.yml";

    // Create default config if it doesn't exist
    if !Path::new(config_file_name).exists() {
        let default_config_yaml = serde_yaml::to_string(&config::Config::default()).unwrap();
        fs::write(config_file_name, default_config_yaml).unwrap();
    }

    // Read yaml config file
    let config_file = std::fs::File::open(config_file_name).unwrap();
    let config: config::Config = serde_yaml::from_reader(config_file).unwrap();

    // Read port from config
    let port: u16 = config.server.http_listen_port;

    // Read instances from config
    let instances: Vec<String> = config.instance_info;

    // TODO @Shinigami92 2022-11-20: Get rid of unsafe
    unsafe {
        INSTANCES.init();
        INSTANCES.extend(instances);
    }

    // Read accounts from config
    let accounts: Vec<(String, String)> = config.accounts;

    // TODO @Shinigami92 2022-11-21: Get rid of unsafe
    unsafe {
        ACCOUNTS.init();
        ACCOUNTS.extend(accounts);
    }

    let routes = warp::get().and(warp::path("metrics").and_then(metrics));

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
