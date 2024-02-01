use async_trait::async_trait;
use octocrab::Octocrab;

use crate::domain::model::repository::{
    AutolinkReference, AutolinkReferenceResponse, RepositoryResponse,
};
use crate::domain::service::github_service::{GitHubService, GitHubServiceError};

#[derive(Clone)]
pub struct OctocrabGitHubService {
    octocrab: Octocrab,
}

impl OctocrabGitHubService {
    pub fn new(octocrab: Octocrab) -> Self {
        Self { octocrab }
    }
}

#[async_trait]
impl GitHubService for OctocrabGitHubService {
    async fn create_repository(
        &self,
        _full_name: &str,
    ) -> Result<RepositoryResponse, GitHubServiceError> {
        todo!()
    }

    async fn get_repository(
        &self,
        full_name: &str,
    ) -> Result<Option<RepositoryResponse>, GitHubServiceError> {
        log::trace!("get_repository: {}", full_name);
        let repository: Result<RepositoryResponse, octocrab::Error> = self
            .octocrab
            .get(format!("/repos/{full_name}"), None::<&()>)
            .await;
        match repository {
            Ok(repository) => Ok(Some(repository)),
            Err(e) => {
                log::error!("get_repository error: {:#?}", e);
                Ok(None)
            }
        }
    }

    async fn update_repository(
        &self,
        full_name: &str,
        repository: &RepositoryResponse,
    ) -> Result<RepositoryResponse, GitHubServiceError> {
        log::trace!("update_repository: {:#?}", &serde_json::json!(repository));
        let repository: Result<RepositoryResponse, octocrab::Error> = self
            .octocrab
            .patch(
                format!("/repos/{full_name}"),
                Some(&serde_json::json!(repository)),
            )
            .await;
        log::debug!("repository: {:#?}", repository);
        self.get_repository(full_name)
            .await
            .map(|r| r.ok_or(GitHubServiceError::Error))?
    }

    async fn get_autolink_references(
        &self,
        full_name: &str,
    ) -> Result<Vec<AutolinkReferenceResponse>, GitHubServiceError> {
        log::trace!("get_autolink_references: {}", full_name);
        let autolink_references: Result<Vec<AutolinkReferenceResponse>, octocrab::Error> = self
            .octocrab
            .get(format!("/repos/{full_name}/autolinks"), None::<&()>)
            .await;
        match autolink_references {
            Ok(autolink_references) => Ok(autolink_references),
            Err(_e) => Ok(vec![]),
        }
    }

    async fn add_autolink_references(
        &self,
        full_name: &str,
        autolink_reference: &AutolinkReference,
    ) -> Result<AutolinkReference, GitHubServiceError> {
        log::trace!("add_autolink_references: {:#?}", &autolink_reference);
        let autolink: Result<AutolinkReference, octocrab::Error> = self
            .octocrab
            .post(
                format!("/repos/{full_name}/autolinks"),
                Some(&serde_json::json!(autolink_reference)),
            )
            .await;
        autolink.map_err(|_| GitHubServiceError::Error)
    }

    async fn delete_autolink_references(
        &self,
        full_name: &str,
        autolink_reference_id: u64,
    ) -> Result<(), GitHubServiceError> {
        log::trace!("update_autolink_references: {:#?}", autolink_reference_id);
        self.octocrab
            ._delete(
                format!("/repos/{full_name}/autolinks/{autolink_reference_id}"),
                None::<&()>,
            )
            .await
            .map(|_| ())
            .map_err(|_| GitHubServiceError::Error)
    }

    async fn archive_repository(&self, _full_name: &str) -> Result<(), GitHubServiceError> {
        Ok(())
    }
}
