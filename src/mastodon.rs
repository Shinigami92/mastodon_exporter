use serde::Deserialize;

/// Information about registering for this website.
///
/// [docs.joinmastodon.org/entities/Instance/#registrations](https://docs.joinmastodon.org/entities/Instance/#registrations)
#[derive(Deserialize)]
pub struct InstanceRegistrations {
    /// Whether registrations are enabled.
    ///
    /// [docs.joinmastodon.org/entities/Instance/#registrations-enabled](https://docs.joinmastodon.org/entities/Instance/#registrations-enabled)
    pub enabled: bool,

    /// Whether registrations require moderator approval.
    ///
    /// [docs.joinmastodon.org/entities/Instance/#approval_required](https://docs.joinmastodon.org/entities/Instance/#approval_required)
    pub approval_required: bool,
}

/// Represents the software instance of Mastodon running on this domain.
///
/// [docs.joinmastodon.org/entities/Instance](https://docs.joinmastodon.org/entities/Instance)
#[derive(Deserialize)]
pub struct InstanceResponse {
    /// The domain name of the instance.
    ///
    /// [docs.joinmastodon.org/entities/Instance/#domain](https://docs.joinmastodon.org/entities/Instance/#domain)
    pub domain: String,

    /// The title of the website.
    ///
    /// [docs.joinmastodon.org/entities/Instance/#title](https://docs.joinmastodon.org/entities/Instance/#title)
    pub title: String,

    /// The version of Mastodon installed on the instance.
    ///
    /// [docs.joinmastodon.org/entities/Instance/#version](https://docs.joinmastodon.org/entities/Instance/#version)
    pub version: String,

    /// Information about registering for this website.
    ///
    /// [docs.joinmastodon.org/entities/Instance/#registrations](https://docs.joinmastodon.org/entities/Instance/#registrations)
    pub registrations: InstanceRegistrations,
}

/// Represents a user of Mastodon and their associated profile.
///
/// [docs.joinmastodon.org/entities/Account](https://docs.joinmastodon.org/entities/Account)
#[derive(Deserialize)]
pub struct AccountResponse {
    /// The username of the account, not including domain.
    ///
    /// [docs.joinmastodon.org/entities/Account/#username](https://docs.joinmastodon.org/entities/Account/#username)
    pub username: String,

    /// The reported followers of this profile.
    ///
    /// [docs.joinmastodon.org/entities/Account/#followers_count](https://docs.joinmastodon.org/entities/Account/#followers_count)
    pub followers_count: i64,

    /// The reported follows of this profile.
    ///
    /// [docs.joinmastodon.org/entities/Account/#following_count](https://docs.joinmastodon.org/entities/Account/#following_count)
    pub following_count: i64,

    /// How many statuses are attached to this account.
    ///
    /// [docs.joinmastodon.org/entities/Account/#statuses_count](https://docs.joinmastodon.org/entities/Account/#statuses_count)
    pub statuses_count: i64,

    /// When the most recent status was posted.
    ///
    /// String (ISO 8601 Date), or null if no statuses.
    ///
    /// [docs.joinmastodon.org/entities/Account/#last_status_at](https://docs.joinmastodon.org/entities/Account/#last_status_at)
    pub last_status_at: Option<String>,
}
