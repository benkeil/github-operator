use async_trait::async_trait;

use crate::domain::model::autolink_reference::{
    AutolinkReferenceRequest, AutolinkReferenceResponse,
};
use crate::domain::model::repository::RepositoryResponse;
use crate::ControllerError;

#[async_trait]
pub trait GitHubService {
    async fn create_repository(
        &self,
        full_name: &str,
        repository: &RepositoryResponse,
    ) -> Result<RepositoryResponse, ControllerError>;
    async fn get_repository(
        &self,
        full_name: &str,
    ) -> Result<Option<RepositoryResponse>, ControllerError>;
    async fn update_repository(
        &self,
        full_name: &str,
        repository: &RepositoryResponse,
    ) -> Result<RepositoryResponse, ControllerError>;
    async fn archive_repository(&self, full_name: &str) -> Result<(), ControllerError>;
    async fn get_autolink_references(
        &self,
        full_name: &str,
    ) -> Result<Vec<AutolinkReferenceResponse>, ControllerError>;
    async fn get_autolink_reference(
        &self,
        full_name: &str,
        id: &u32,
    ) -> Result<AutolinkReferenceResponse, ControllerError>;
    async fn add_autolink_reference(
        &self,
        full_name: &str,
        autolink_reference: &AutolinkReferenceRequest,
    ) -> Result<AutolinkReferenceResponse, ControllerError>;
    async fn delete_autolink_references(
        &self,
        full_name: &str,
        autolink_reference_id: &u32,
    ) -> Result<(), ControllerError>;
    async fn get_team_permission(
        &self,
        full_name: &str,
        full_team_name: &str,
    ) -> Result<Option<String>, ControllerError>;
    async fn update_team_permission(
        &self,
        full_name: &str,
        full_team_name: &str,
        role_name: &str,
    ) -> Result<(), ControllerError>;
    async fn delete_team_permission(
        &self,
        full_name: &str,
        full_team_name: &str,
    ) -> Result<(), ControllerError>;
}
