#[macro_use]
extern crate lazy_static;

use prometheus::{Encoder, IntGaugeVec, Opts, Registry, TextEncoder};
use serde_json::Value;
use warp::Filter;

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
    let ratelimit_reset = response
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
    let body = response.json::<Value>().await?;

    // Collect instance info
    let info_labels = [
        instance,
        body["domain"].as_str().unwrap(),
        body["title"].as_str().unwrap(),
        body["version"].as_str().unwrap(),
    ];
    println!("Instance info: {:?}", info_labels);
    MASTODON_INFO.with_label_values(&info_labels).set(1);

    // Collect registrations_enabled value
    let registrations_enabled: i64 = body["registrations"]["enabled"].as_bool().unwrap() as i64;
    println!(
        "{}: Registrations enabled: {:?}",
        instance, registrations_enabled
    );
    MASTODON_REGISTRATIONS_ENABLED
        .with_label_values(&[instance])
        .set(registrations_enabled);

    // Collect registrations_approval_required value
    let registrations_approval_required = body["registrations"]["approval_required"]
        .as_bool()
        .unwrap() as i64;
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
    let instances = vec!["mas.to", "mastodon.social"];

    let tasks = instances
        .iter()
        .map(|instance| tokio::spawn(async { collect_instance(instance).await }))
        .collect::<Vec<_>>();

    for task in tasks {
        task.await?.unwrap();
    }

    Ok(())
}

async fn metrics() -> Result<impl warp::Reply, warp::Rejection> {
    collect_instances().await.unwrap();

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

    let routes = warp::get().and(warp::path("metrics").and_then(metrics));

    warp::serve(routes).run(([127, 0, 0, 1], 9498)).await;
}
