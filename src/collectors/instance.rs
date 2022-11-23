use crate::{
    mastodon, INSTANCES, MASTODON_INFO, MASTODON_RATELIMIT_REMAINING, MASTODON_RATELIMIT_RESET,
    MASTODON_REGISTRATIONS_APPROVAL_REQUIRED, MASTODON_REGISTRATIONS_ENABLED,
};

pub async fn collect_instance(instance: &str) -> Result<(), reqwest::Error> {
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

    if let Err(err) = &response.error_for_status_ref() {
        println!("Error: {} {}", instance, err);
        return Ok(());
    }

    // Collect response body data
    let body = response.json::<mastodon::InstanceResponse>().await.unwrap();

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

pub async fn collect_instances() -> Result<(), tokio::task::JoinError> {
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
