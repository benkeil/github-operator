use octocrab::{
    DefaultOctocrabBuilderConfig, NoAuth, NoSvc, NotLayerReady, Octocrab, OctocrabBuilder,
};
use std::time::Duration;

pub trait DurationExtension {
    fn from_minutes(minutes: u64) -> Duration;
}

impl DurationExtension for Duration {
    fn from_minutes(minutes: u64) -> Duration {
        Duration::from_secs(minutes * 60)
    }
}

pub trait OctocrabExtension {
    fn from_env() -> Octocrab;
}

impl OctocrabExtension
    for OctocrabBuilder<NoSvc, DefaultOctocrabBuilderConfig, NoAuth, NotLayerReady>
{
    fn from_env() -> Octocrab {
        Self::default()
            .personal_token(std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN is not set"))
            .build()
            .expect("failed to create github client")
    }
}
