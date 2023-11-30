use async_trait::async_trait;

use crate::domain::model::repository::{Repository, Status};

#[async_trait]
pub trait GitHubService {
    async fn get_repository(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<Option<Repository>, GitHubServiceError>;
    async fn create_repository(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<Repository, GitHubServiceError>;
    async fn set_secret_scanning(
        &self,
        owner: &str,
        name: &str,
        value: Status,
    ) -> Result<(), GitHubServiceError>;
    async fn set_secret_scanning_push_protection(
        &self,
        owner: &str,
        name: &str,
        value: Status,
    ) -> Result<(), GitHubServiceError>;
    async fn set_dependabot_security_updates(
        &self,
        owner: &str,
        name: &str,
        value: Status,
    ) -> Result<(), GitHubServiceError>;
    async fn set_secret_scanning_validity_checks(
        &self,
        owner: &str,
        name: &str,
        value: Status,
    ) -> Result<(), GitHubServiceError>;
    async fn archive_repository(&self, owner: &str, name: &str) -> Result<(), GitHubServiceError>;
}

pub enum GitHubServiceError {}
