use async_trait::async_trait;
use futures::future::join_all;
use itertools::Itertools;
use octocrab::Octocrab;

use crate::domain::model::repository::{AutolinkReference, Repository, RepositoryResponse};
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
                let autolink_references = self
                    .get_autolinks(owner, name)
                    .await
                    .map_err(|_| GitHubServiceError::Error)?;
                Ok(Some(Repository {
                    repository,
                    autolink_references: match autolink_references.len() {
                        x if x > 0 => Some(autolink_references),
                        _ => None,
                    },
                }))
            }
            Err(e) => {
                log::info!("get_repository: {:#?}", e);
                Err(GitHubServiceError::Error)
            }
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
        repository: &RepositoryResponse,
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

    async fn update_autolink_references(
        &self,
        owner: &str,
        name: &str,
        autolink_references: &Vec<AutolinkReference>,
    ) -> Result<Vec<AutolinkReference>, GitHubServiceError> {
        log::trace!("update_autolink_references({}/{})", owner, name);
        let autolink_references_futures =
            autolink_references
                .iter()
                .map(|autolink_reference| async move {
                    let autolink: Result<AutolinkReference, octocrab::Error> = self
                        .octocrab
                        .post(
                            format!("/repos/{owner}/{name}/autolinks"),
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

    async fn archive_repository(&self, owner: &str, name: &str) -> Result<(), GitHubServiceError> {
        Ok(())
    }
}
