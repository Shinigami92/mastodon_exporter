use chrono::NaiveDate;

use crate::{
    mastodon, ACCOUNTS, MASTODON_ACCOUNT_FOLLOWERS_COUNT, MASTODON_ACCOUNT_FOLLOWING_COUNT,
    MASTODON_ACCOUNT_LAST_STATUS_AT, MASTODON_ACCOUNT_STATUSES_COUNT, MASTODON_RATELIMIT_REMAINING,
    MASTODON_RATELIMIT_RESET,
};

pub async fn collect_account(instance: &str, account_id: &str) -> Result<(), reqwest::Error> {
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

    if let Err(err) = &response.error_for_status_ref() {
        if err.status() == Some(reqwest::StatusCode::NOT_FOUND) {
            println!("{}: Account {} not found", instance, account_id);
            return Ok(());
        }

        println!("Error: {} {} {}", instance, account_id, err);
        return Ok(());
    }

    // Collect response body data
    let body = response.json::<mastodon::AccountResponse>().await.unwrap();

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

pub async fn collect_accounts() -> Result<(), tokio::task::JoinError> {
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
