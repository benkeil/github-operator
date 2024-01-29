use std::fmt::{Debug, Display};

use async_trait::async_trait;

use crate::domain::model::repository::{AutolinkReference, Repository, RepositoryResponse};

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
        repository: &RepositoryResponse,
    ) -> Result<Repository, GitHubServiceError>;
    async fn update_autolink_references(
        &self,
        owner: &str,
        name: &str,
        autolink_references: Vec<AutolinkReference>,
    ) -> Result<Vec<AutolinkReference>, GitHubServiceError>;
    async fn archive_repository(&self, owner: &str, name: &str) -> Result<(), GitHubServiceError>;
}

#[derive(PartialEq, Debug)]
pub enum GitHubServiceError {
    Error,
    NotFound,
}
