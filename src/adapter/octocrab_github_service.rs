use async_trait::async_trait;
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
        Ok(Some(Repository {
            repository: repository.unwrap(),
            autolink_references: None,
        }))
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


    async fn archive_repository(&self, owner: &str, name: &str) -> Result<(), GitHubServiceError> {
        todo!()
    }
}
