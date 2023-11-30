use async_trait::async_trait;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

use crate::domain::model::repository::{Repository, SecurityAndAnalysis, Status};
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

#[async_trait]
impl GitHubService for OctocrabGitHubService {
    async fn get_repository(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<Option<Repository>, GitHubServiceError> {
        let repository: Result<RepositoryResponse, octocrab::Error> = self
            .octocrab
            .get(format!("/repos/{owner}/{name}"), None::<&()>)
            .await;
        log::debug!("repository: {:#?}", repository);
        match repository {
            Ok(repository) => Ok(Some(repository.into())),
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

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
struct RepositoryResponse {
    pub full_name: String,
    pub security_and_analysis: Option<SecurityAndAnalysisResponse>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
struct SecurityAndAnalysisResponse {
    pub secret_scanning: SecurityAndAnalysisStatusResponse,
    pub secret_scanning_push_protection: SecurityAndAnalysisStatusResponse,
    pub dependabot_security_updates: SecurityAndAnalysisStatusResponse,
    pub secret_scanning_validity_checks: SecurityAndAnalysisStatusResponse,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct SecurityAndAnalysisStatusResponse {
    pub status: StatusResponse,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum StatusResponse {
    Enabled,
    Disabled,
}

impl From<RepositoryResponse> for Repository {
    fn from(repository_response: RepositoryResponse) -> Self {
        Self {
            full_name: repository_response.full_name,
            security_and_analysis: repository_response.security_and_analysis.map(
                |security_and_analysis| {
                    let secret_scanning = security_and_analysis.secret_scanning.status.into();
                    let secret_scanning_push_protection = security_and_analysis
                        .secret_scanning_push_protection
                        .status
                        .into();
                    let dependabot_security_updates = security_and_analysis
                        .dependabot_security_updates
                        .status
                        .into();
                    let secret_scanning_validity_checks = security_and_analysis
                        .secret_scanning_validity_checks
                        .status
                        .into();
                    SecurityAndAnalysis {
                        secret_scanning,
                        secret_scanning_push_protection,
                        dependabot_security_updates,
                        secret_scanning_validity_checks,
                    }
                },
            ),
        }
    }
}

impl From<StatusResponse> for Status {
    fn from(status: StatusResponse) -> Self {
        match status {
            StatusResponse::Enabled => Self::Enabled,
            StatusResponse::Disabled => Self::Disabled,
        }
    }
}
