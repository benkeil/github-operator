use async_trait::async_trait;
use octocrab::{Error, GitHubError, Octocrab};

use crate::domain::model::autolink_reference::{
    AutolinkReferenceRequest, AutolinkReferenceResponse,
};
use crate::domain::model::repository::RepositoryResponse;
use crate::domain::service::github_service::GitHubService;
use crate::ControllerError;

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
    ) -> Result<RepositoryResponse, ControllerError> {
        todo!()
    }

    async fn get_repository(
        &self,
        full_name: &str,
    ) -> Result<Option<RepositoryResponse>, ControllerError> {
        log::trace!("get_repository: {}", full_name);
        let repository: Result<RepositoryResponse, octocrab::Error> = self
            .octocrab
            .get(format!("/repos/{full_name}"), None::<&()>)
            .await;
        match repository {
            Ok(repository) => Ok(Some(repository)),
            Err(e) => {
                // TODO: handle 404
                log::error!("get_repository error: {:#?}", e);
                Ok(None)
            }
        }
    }

    async fn update_repository(
        &self,
        full_name: &str,
        repository: &RepositoryResponse,
    ) -> Result<RepositoryResponse, ControllerError> {
        log::trace!("update_repository: {:#?}", &serde_json::json!(repository));
        let repository: Result<RepositoryResponse, octocrab::Error> = self
            .octocrab
            .patch(
                format!("/repos/{full_name}"),
                Some(&serde_json::json!(repository)),
            )
            .await;
        log::debug!("repository: {:#?}", repository);
        let repository = self
            .get_repository(full_name)
            .await
            .map(|response| response.expect("repository should be existent"))?;
        Ok(repository)
    }

    async fn get_autolink_references(
        &self,
        full_name: &str,
    ) -> Result<Vec<AutolinkReferenceResponse>, ControllerError> {
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

    async fn get_autolink_reference(
        &self,
        full_name: &str,
        id: &u32,
    ) -> Result<AutolinkReferenceResponse, ControllerError> {
        log::trace!("get_autolink_reference: {}/{}", full_name, id);
        let autolink_reference: Result<AutolinkReferenceResponse, octocrab::Error> = self
            .octocrab
            .get(format!("/repos/{full_name}/autolinks/{id}"), None::<&()>)
            .await;
        autolink_reference.map_err(ControllerError::GitHubError)
    }

    async fn add_autolink_reference(
        &self,
        full_name: &str,
        autolink_reference: &AutolinkReferenceRequest,
    ) -> Result<AutolinkReferenceResponse, ControllerError> {
        log::trace!("add_autolink_reference: {:#?}", &autolink_reference);
        let autolink: Result<AutolinkReferenceResponse, octocrab::Error> = self
            .octocrab
            .post(
                format!("/repos/{full_name}/autolinks"),
                Some(&serde_json::json!(autolink_reference)),
            )
            .await;
        // TODO junge junge...
        if let Err(octocrab::Error::GitHub { source, .. }) = &autolink {
            if let Some(errors) = &source.errors {
                if errors.len() == 1 {
                    if let Some(message) = errors[0].get("code") {
                        if message == "already_exists" {
                            return Err(ControllerError::AlreadyExists);
                        }
                    }
                }
            }
            log::error!("{:#?}", source);
        }
        autolink.map_err(ControllerError::GitHubError)
    }

    async fn delete_autolink_references(
        &self,
        full_name: &str,
        autolink_reference_id: &u32,
    ) -> Result<(), ControllerError> {
        log::trace!("delete_autolink_references: {:#?}", autolink_reference_id);
        self.octocrab
            ._delete(
                format!("/repos/{full_name}/autolinks/{autolink_reference_id}"),
                None::<&()>,
            )
            .await
            .map(|_| ())
            .map_err(ControllerError::GitHubError)
    }

    async fn archive_repository(&self, _full_name: &str) -> Result<(), ControllerError> {
        Ok(())
    }
}

fn map_octocrab_error<T>(result: Result<T, octocrab::Error>) -> Result<T, ControllerError> {
    match result {
        Ok(result) => Ok(result),
        Err(Error::GitHub {
            source:
                GitHubError {
                    errors: Some(errors),
                    ..
                },
            ..
        }) => {
            if errors.len() == 1 {
                if let Some(serde_json::Value::String(message)) = errors[0].get("code") {
                    if message == "already_exists" {
                        return Err(ControllerError::AlreadyExists);
                    }
                }
            }
            Err(ControllerError::GitHubError(source))
        }
        Err(e) => Err(ControllerError::GitHubError(e)),
    }
}
