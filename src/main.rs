#[macro_use]
extern crate lazy_mut;
#[macro_use]
extern crate lazy_static;
extern crate serde_derive;
extern crate serde_yaml;

use std::{fs, path::Path};

use chrono::NaiveDate;
use prometheus::{Encoder, IntGaugeVec, Opts, Registry, TextEncoder};
use serde_derive::{Deserialize, Serialize};
use warp::Filter;

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

async fn collect_instance(instance: &str) -> Result<(), reqwest::Error> {
    let url = format!("https://{}/api/v2/instance", instance);

    println!("Collecting instance {}", instance);

    let response = reqwest::get(url).await?;

    // Collect x-ratelimit-remaining from header
    let ratelimit_remaining: i64 = response
        .headers()
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    println!("{}: Ratelimit remaining: {}", instance, ratelimit_remaining);
    MASTODON_RATELIMIT_REMAINING
        .with_label_values(&[instance])
        .set(ratelimit_remaining);

    // Collect x-ratelimit-reset from header
    let ratelimit_reset: i64 = response
        .headers()
        .get("x-ratelimit-reset")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap()
        .timestamp();
    println!("{}: Ratelimit reset: {}", instance, ratelimit_reset);
    MASTODON_RATELIMIT_RESET
        .with_label_values(&[instance])
        .set(ratelimit_reset);

    // Collect response body data
    let body = response.json::<mastodon::InstanceResponse>().await?;

    // Collect instance info
    let info_labels = [instance, &body.domain, &body.title, &body.version];
    println!("Instance info: {:?}", info_labels);
    MASTODON_INFO.with_label_values(&info_labels).set(1);

    // Collect registrations_enabled value
    let registrations_enabled = i64::from(body.registrations.enabled);
    println!(
        "{}: Registrations enabled: {:?}",
        instance, registrations_enabled
    );
    MASTODON_REGISTRATIONS_ENABLED
        .with_label_values(&[instance])
        .set(registrations_enabled);

    // Collect registrations_approval_required value
    let registrations_approval_required = i64::from(body.registrations.approval_required);
    println!(
        "{}: Registrations approval required: {:?}",
        instance, registrations_approval_required
    );
    MASTODON_REGISTRATIONS_APPROVAL_REQUIRED
        .with_label_values(&[instance])
        .set(registrations_approval_required);

    Ok(())
}

async fn collect_instances() -> Result<(), tokio::task::JoinError> {
    // TODO @Shinigami92 2022-11-20: Get rid of unsafe
    unsafe {
        let tasks = INSTANCES
            .iter()
            .map(|instance| tokio::spawn(async { collect_instance(instance).await }))
            .collect::<Vec<_>>();

        for task in tasks {
            task.await?.ok();
        }
    }

    Ok(())
}

async fn collect_account(instance: &str, account_id: &str) -> Result<(), reqwest::Error> {
    let url = format!("https://{}/api/v1/accounts/{}", instance, account_id);

    println!("Collecting account {}@{}", account_id, instance);

    let response = reqwest::get(url).await?;

    // Collect x-ratelimit-remaining from header
    let ratelimit_remaining: i64 = response
        .headers()
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    println!("{}: Ratelimit remaining: {}", instance, ratelimit_remaining);
    MASTODON_RATELIMIT_REMAINING
        .with_label_values(&[instance])
        .set(ratelimit_remaining);

    // Collect x-ratelimit-reset from header
    let ratelimit_reset: i64 = response
        .headers()
        .get("x-ratelimit-reset")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap()
        .timestamp();
    println!("{}: Ratelimit reset: {}", instance, ratelimit_reset);
    MASTODON_RATELIMIT_RESET
        .with_label_values(&[instance])
        .set(ratelimit_reset);

    // Collect response body data
    let body = response.json::<mastodon::AccountResponse>().await?;

    // TODO @Shinigami92 2022-11-21: Handle case when account is not found
    let username = &body.username;

    // Collect account info
    let info_labels = [instance, account_id, username];
    println!("Account info: {:?}", info_labels);

    // Collect account followers count
    let followers_count = body.followers_count;
    println!(
        "@{}@{}: Followers count: {}",
        username, instance, followers_count
    );
    MASTODON_ACCOUNT_FOLLOWERS_COUNT
        .with_label_values(&info_labels)
        .set(followers_count);

    // Collect account following count
    let following_count = body.following_count;
    println!(
        "@{}@{}: Following count: {}",
        username, instance, following_count
    );
    MASTODON_ACCOUNT_FOLLOWING_COUNT
        .with_label_values(&info_labels)
        .set(following_count);

    // Collect account statuses count
    let statuses_count = body.statuses_count;
    println!(
        "@{}@{}: Statuses count: {}",
        username, instance, statuses_count
    );
    MASTODON_ACCOUNT_STATUSES_COUNT
        .with_label_values(&info_labels)
        .set(statuses_count);

    // Collect account last status at
    if let Some(last_status_at) = body.last_status_at {
        let last_status_at: i64 = NaiveDate::parse_from_str(&last_status_at, "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .timestamp();

        println!(
            "@{}@{}: Last status at: {}",
            username, instance, last_status_at
        );
        MASTODON_ACCOUNT_LAST_STATUS_AT
            .with_label_values(&info_labels)
            .set(last_status_at);
    }

    Ok(())
}

async fn collect_accounts() -> Result<(), tokio::task::JoinError> {
    // TODO @Shinigami92 2022-11-21: Get rid of unsafe
    unsafe {
        let tasks = ACCOUNTS
            .iter()
            .map(|(instance, account_id)| {
                tokio::spawn(async { collect_account(instance, account_id).await })
            })
            .collect::<Vec<_>>();

        for task in tasks {
            task.await?.ok();
        }
    }

    Ok(())
}

async fn metrics() -> Result<impl warp::Reply, warp::Rejection> {
    collect_instances().await.ok();
    collect_accounts().await.ok();

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    Ok(String::from_utf8(buffer).unwrap())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ServerConfig {
    http_listen_port: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    server: ServerConfig,
    instance_info: Vec<String>,
    accounts: Vec<(String, String)>,
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
        let default_config = Config {
            server: ServerConfig {
                http_listen_port: 9498,
            },
            instance_info: vec!["mas.to".to_string(), "mastodon.social".to_string()],
            accounts: vec![],
        };
        let default_config_yaml = serde_yaml::to_string(&default_config).unwrap();
        fs::write(config_file_name, default_config_yaml).unwrap();
    }

    // Read yaml config file
    let config_file = std::fs::File::open(config_file_name).unwrap();
    let config: Config = serde_yaml::from_reader(config_file).unwrap();

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
