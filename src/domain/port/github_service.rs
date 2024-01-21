use std::fmt::{Debug, Display};

use async_trait::async_trait;
use serde::Serialize;

use crate::domain::model::github_repository::Status;
use crate::domain::model::repository::Repository;
use crate::domain::model::update_repository::UpdateRepository;

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
    async fn update_repository(
        &self,
        owner: &str,
        name: &str,
        repository: &UpdateRepository,
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

#[derive(PartialEq, Debug)]
pub enum GitHubServiceError {
    Error,
}
