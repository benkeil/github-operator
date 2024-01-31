use async_trait::async_trait;
use futures::future::join_all;
use octocrab::Octocrab;

use crate::domain::model::repository::{AutolinkReference, RepositoryResponse};
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
        log::trace!("get_repository({})", full_name);
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
    ) -> Result<Vec<AutolinkReference>, GitHubServiceError> {
        log::trace!("get_autolink_references({})", full_name);
        let autolink_references: Result<Vec<AutolinkReference>, octocrab::Error> = self
            .octocrab
            .get(format!("/repos/{full_name}/autolinks"), None::<&()>)
            .await;
        match autolink_references {
            Ok(autolink_references) => Ok(autolink_references),
            Err(_e) => Ok(vec![]),
        }
    }

    async fn update_autolink_references(
        &self,
        full_name: &str,
        autolink_references: &[AutolinkReference],
    ) -> Result<Vec<AutolinkReference>, GitHubServiceError> {
        // TODO: We need to handle each item step by step, not all at once (create, delete).
        log::trace!("update_autolink_references({})", full_name);
        let autolink_references_futures =
            autolink_references
                .iter()
                .map(|autolink_reference| async move {
                    let autolink: Result<AutolinkReference, octocrab::Error> = self
                        .octocrab
                        .post(
                            format!("/repos/{full_name}/autolinks"),
                            Some(&serde_json::json!(autolink_reference)),
                        )
                        .await;
                    log::debug!("==> autolink: {:#?}", autolink);
                    autolink.map_err(|_| GitHubServiceError::Error)
                });
        let results = join_all(autolink_references_futures).await;
        let result: Result<Vec<AutolinkReference>, GitHubServiceError> =
            results.into_iter().collect();
        result
    }

    async fn archive_repository(&self, _full_name: &str) -> Result<(), GitHubServiceError> {
        Ok(())
    }
}
