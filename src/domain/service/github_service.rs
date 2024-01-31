use std::fmt::Debug;

use async_trait::async_trait;

use crate::domain::model::repository::{AutolinkReference, RepositoryResponse};

#[async_trait]
pub trait GitHubService {
    async fn create_repository(
        &self,
        full_name: &str,
    ) -> Result<RepositoryResponse, GitHubServiceError>;
    async fn get_repository(
        &self,
        full_name: &str,
    ) -> Result<Option<RepositoryResponse>, GitHubServiceError>;
    async fn update_repository(
        &self,
        full_name: &str,
        repository: &RepositoryResponse,
    ) -> Result<RepositoryResponse, GitHubServiceError>;
    async fn get_autolink_references(
        &self,
        full_name: &str,
    ) -> Result<Vec<AutolinkReference>, GitHubServiceError>;
    async fn update_autolink_references(
        &self,
        full_name: &str,
        autolink_references: &[AutolinkReference],
    ) -> Result<Vec<AutolinkReference>, GitHubServiceError>;
    async fn archive_repository(&self, full_name: &str) -> Result<(), GitHubServiceError>;
}

#[derive(PartialEq, Debug)]
pub enum GitHubServiceError {
    Error,
    NotFound,
}
