use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use tracing::instrument;
use ureq::Request;

use crate::domain::model::autolink_reference::{
    AutolinkReferenceRequest, AutolinkReferenceResponse,
};
use crate::domain::model::repository::RepositoryResponse;
use crate::domain::service::github_service::GitHubService;
use crate::ControllerError;

#[derive(Clone, Debug)]
pub struct HttpGithubService {
    client: ureq::Agent,
    github_token: String,
}

// as an alternate: https://docs.rs/reqwest/latest/reqwest/

impl HttpGithubService {
    const HOST: &'static str = "https://api.github.com";

    pub fn from_env() -> Self {
        let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN is not set");
        Self::new(github_token)
    }

    pub fn new(github_token: String) -> Self {
        let client = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();
        Self {
            client,
            github_token,
        }
    }

    fn url(&self, path: String) -> String {
        format!("{}{}", Self::HOST, path.as_str())
    }

    fn get(&self, path: String) -> Request {
        self.client
            .get(self.url(path).as_str())
            .set("Authorization", &format!("Bearer {}", self.github_token))
    }

    fn delete(&self, path: String) -> Request {
        self.client
            .delete(self.url(path).as_str())
            .set("Authorization", &format!("Bearer {}", self.github_token))
    }

    fn post(&self, path: String) -> Request {
        self.client
            .post(self.url(path).as_str())
            .set("Authorization", &format!("Bearer {}", self.github_token))
    }

    fn put(&self, path: String) -> Request {
        self.client
            .put(self.url(path).as_str())
            .set("Authorization", &format!("Bearer {}", self.github_token))
    }

    fn box_error(error: ureq::Error) -> ControllerError {
        ControllerError::HttpError(Box::new(error))
    }
}

#[async_trait]
impl GitHubService for HttpGithubService {
    #[instrument]
    async fn create_repository(
        &self,
        full_name: &str,
        repository: &RepositoryResponse,
    ) -> Result<RepositoryResponse, ControllerError> {
        self.post(format!("/repos/{full_name}"))
            .send_json(ureq::json!(repository))
            .map_err(Self::box_error)?
            .into_json()
            .map_err(ControllerError::IoError)
    }

    #[instrument]
    async fn get_repository(
        &self,
        full_name: &str,
    ) -> Result<Option<RepositoryResponse>, ControllerError> {
        let result = self.get(format!("/repos/{full_name}")).call();
        match result {
            Ok(response) => Ok(Some(
                response.into_json().map_err(ControllerError::IoError)?,
            )),
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(e) => Err(Self::box_error(e)),
        }
    }

    #[instrument]
    async fn update_repository(
        &self,
        full_name: &str,
        repository: &RepositoryResponse,
    ) -> Result<RepositoryResponse, ControllerError> {
        let result = self
            .post(format!("/repos/{full_name}"))
            .send_json(ureq::json!(repository));
        match result {
            Ok(response) => Ok(response.into_json().map_err(ControllerError::IoError)?),
            Err(ureq::Error::Status(404, _)) => Err(ControllerError::NotFound),
            Err(e) => Err(Self::box_error(e)),
        }
    }

    #[instrument]
    async fn archive_repository(&self, _full_name: &str) -> Result<(), ControllerError> {
        Ok(())
    }

    #[instrument]
    async fn get_autolink_references(
        &self,
        full_name: &str,
    ) -> Result<Vec<AutolinkReferenceResponse>, ControllerError> {
        self.get(format!("/repos/{full_name}/autolinks"))
            .call()
            .map_err(Self::box_error)?
            .into_json()
            .map_err(ControllerError::IoError)
    }

    #[instrument]
    async fn get_autolink_reference(
        &self,
        full_name: &str,
        id: &u32,
    ) -> Result<AutolinkReferenceResponse, ControllerError> {
        let result = self
            .get(format!("/repos/{full_name}/autolinks/{id}"))
            .call();
        match result {
            Ok(response) => Ok(response.into_json().map_err(ControllerError::IoError)?),
            Err(ureq::Error::Status(404, _)) => Err(ControllerError::NotFound),
            Err(e) => Err(Self::box_error(e)),
        }
    }

    #[instrument]
    async fn add_autolink_reference(
        &self,
        full_name: &str,
        autolink_reference: &AutolinkReferenceRequest,
    ) -> Result<AutolinkReferenceResponse, ControllerError> {
        self.post(format!("/repos/{full_name}/autolinks"))
            .send_json(ureq::json!(autolink_reference))
            .map_err(Self::box_error)?
            .into_json()
            .map_err(ControllerError::IoError)
    }

    #[instrument]
    async fn delete_autolink_references(
        &self,
        full_name: &str,
        autolink_reference_id: &u32,
    ) -> Result<(), ControllerError> {
        let result = self
            .delete(format!(
                "/repos/{full_name}/autolinks/{autolink_reference_id}"
            ))
            .call()
            .map(|_| ());
        match result {
            Ok(()) => Ok(()),
            Err(ureq::Error::Status(404, _)) => Ok(()),
            Err(e) => Err(Self::box_error(e)),
        }
    }

    #[instrument]
    async fn get_team_permission(
        &self,
        full_name: &str,
        full_team_name: &str,
    ) -> Result<Option<String>, ControllerError> {
        let (org, team_slug) = full_team_name
            .split_once('/')
            .expect("team name should be valid");
        let result = self
            .get(format!("/orgs/{org}/teams/{team_slug}/repos/{full_name}"))
            .set("Accept", "application/vnd.github.v3.repository+json")
            .call();
        match result {
            Ok(response) => {
                let repository: TeamPermissionRepositoryResponse =
                    response.into_json().map_err(ControllerError::IoError)?;
                Ok(Some(repository.role_name))
            }
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(error) => Err(Self::box_error(error)),
        }
    }

    #[instrument]
    async fn update_team_permission(
        &self,
        full_name: &str,
        full_team_name: &str,
        role_name: &str,
    ) -> Result<(), ControllerError> {
        let (org, team_slug) = full_team_name
            .split_once('/')
            .expect("team name should be valid");
        self.put(format!("/orgs/{org}/teams/{team_slug}/repos/{full_name}"))
            .set("Accept", "application/vnd.github.v3.repository+json")
            .send_json(ureq::json!({"permission": role_name}))
            .map(|_| ())
            .map_err(Self::box_error)
    }

    #[instrument]
    async fn delete_team_permission(
        &self,
        full_name: &str,
        full_team_name: &str,
    ) -> Result<(), ControllerError> {
        let (org, team_slug) = full_team_name
            .split_once('/')
            .expect("team name should be valid");
        self.delete(format!("/orgs/{org}/teams/{team_slug}/repos/{full_name}"))
            .set("Accept", "application/vnd.github.v3.repository+json")
            .call()
            .map(|_| ())
            .map_err(Self::box_error)
    }
}

#[derive(Deserialize)]
struct TeamPermissionRepositoryResponse {
    role_name: String,
}

#[cfg(test)]
mod tests {
    use crate::adapter::http_github_service::HttpGithubService;
    use crate::domain::service::github_service::GitHubService;

    #[tokio::test]
    async fn test() {
        let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN is not set");
        let github_service = HttpGithubService::new(github_token);
        let result = github_service
            .get_autolink_references("otto-ec/pdh-da_alarm-notification")
            .await;
        println!("{:#?}", result)
    }
}
