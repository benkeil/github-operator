use crate::domain::model::github_repository::{ActionsSettings, AutolinkReference, Status};
use async_trait::async_trait;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

use crate::domain::model::repository::{Repository, SecurityAndAnalysis};
use crate::domain::model::update_repository::UpdateRepository;
use crate::domain::port::github_service::{GitHubService, GitHubServiceError};

#[derive(Clone)]
pub struct OctocrabGitHubService {
    octocrab: Octocrab,
}

impl OctocrabGitHubService {
    pub fn new(octocrab: Octocrab) -> Self {
        Self { octocrab }
    }
}

impl OctocrabGitHubService {
    async fn get_autolinks(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<Vec<AutolinkReference>, GitHubServiceError> {
        log::trace!("get_autolinks({}/{})", owner, name);
        let autolinks: Result<Vec<AutolinkReference>, octocrab::Error> = self
            .octocrab
            .get(format!("/repos/{owner}/{name}/autolinks"), None::<&()>)
            .await;
        log::debug!("autolinks: {:#?}", autolinks);
        match autolinks {
            Ok(autolinks) => Ok(autolinks),
            Err(e) => Ok(vec![]),
        }
    }
}

#[async_trait]
impl GitHubService for OctocrabGitHubService {
    async fn get_repository(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<Option<Repository>, GitHubServiceError> {
        log::trace!("get_repository({}/{})", owner, name);
        let repository: Result<RepositoryResponse, octocrab::Error> = self
            .octocrab
            .get(format!("/repos/{owner}/{name}"), None::<&()>)
            .await;
        match repository {
            Ok(repository) => {
                log::debug!("repository: {:#?}", repository);
                let autolinks = self.get_autolinks(owner, name).await?;
                Ok(Some(Repository {
                    full_name: repository.full_name,
                    security_and_analysis: repository.security_and_analysis.into(),
                    autolink_references: autolinks,
                    delete_branch_on_merge: repository.delete_branch_on_merge,
                    allow_auto_merge: repository.allow_auto_merge,
                    allow_squash_merge: repository.allow_squash_merge,
                    allow_merge_commit: repository.allow_merge_commit,
                    allow_rebase_merge: repository.allow_rebase_merge,
                    allow_update_branch: repository.allow_update_branch,
                }))
            }
            Err(e) => Ok(None),
        }
    }

    async fn create_repository(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<Repository, GitHubServiceError> {
        todo!()
    }

    async fn update_repository(
        &self,
        owner: &str,
        name: &str,
        repository: &UpdateRepository,
    ) -> Result<Repository, GitHubServiceError> {
        log::trace!("update_repository: {:#?}", &serde_json::json!(repository));
        let repository: Result<RepositoryResponse, octocrab::Error> = self
            .octocrab
            .patch(
                format!("/repos/{owner}/{name}"),
                Some(&serde_json::json!(repository)),
            )
            .await;
        log::debug!("repository: {:#?}", repository);
        self.get_repository(owner, name)
            .await
            .map(|r| r.ok_or(GitHubServiceError::Error))?
    }

    async fn set_secret_scanning(
        &self,
        owner: &str,
        name: &str,
        value: Status,
    ) -> Result<(), GitHubServiceError> {
        // todo not sure where it belongs to
        let response = self
            .octocrab
            ._put(
                format!("/repos/{owner}/{name}/private-vulnerability-reporting"),
                None::<&()>,
            )
            .await;
        log::info!("{:#?}", response);
        Ok(())
    }

    async fn set_secret_scanning_push_protection(
        &self,
        owner: &str,
        name: &str,
        value: Status,
    ) -> Result<(), GitHubServiceError> {
        todo!()
    }

    async fn set_dependabot_security_updates(
        &self,
        owner: &str,
        name: &str,
        value: Status,
    ) -> Result<(), GitHubServiceError> {
        todo!()
    }

    async fn set_secret_scanning_validity_checks(
        &self,
        owner: &str,
        name: &str,
        value: Status,
    ) -> Result<(), GitHubServiceError> {
        todo!()
    }

    async fn archive_repository(&self, owner: &str, name: &str) -> Result<(), GitHubServiceError> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RepositoryResponse {
    pub full_name: String,
    pub security_and_analysis: SecurityAndAnalysisResponse,
    // pub actions: ActionsSettings,
    pub delete_branch_on_merge: bool,
    pub allow_auto_merge: bool,
    pub allow_squash_merge: bool,
    pub allow_merge_commit: bool,
    pub allow_rebase_merge: bool,
    pub allow_update_branch: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SecurityAndAnalysisResponse {
    pub secret_scanning: SecurityAndAnalysisStatusResponse,
    pub secret_scanning_push_protection: SecurityAndAnalysisStatusResponse,
    pub dependabot_security_updates: SecurityAndAnalysisStatusResponse,
    pub secret_scanning_validity_checks: SecurityAndAnalysisStatusResponse,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SecurityAndAnalysisStatusResponse {
    pub status: Status,
}

impl From<SecurityAndAnalysisResponse> for SecurityAndAnalysis {
    fn from(value: SecurityAndAnalysisResponse) -> Self {
        SecurityAndAnalysis {
            secret_scanning: value.secret_scanning.status,
            secret_scanning_push_protection: value.secret_scanning_push_protection.status,
            dependabot_security_updates: value.dependabot_security_updates.status,
            secret_scanning_validity_checks: value.secret_scanning_validity_checks.status,
        }
    }
}
