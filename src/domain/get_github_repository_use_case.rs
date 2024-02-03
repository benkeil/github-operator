use crate::domain::model::github_repository_spec::GitHubRepositorySpec;
use crate::domain::model::repository::{AutolinkReference, AutolinkReferenceResponse};
use crate::domain::service::github_service::GitHubService;

pub struct GetGitHubRepositoryUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl GetGitHubRepositoryUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        full_name: &str,
    ) -> Result<Option<GitHubRepositorySpec>, GetGitHubRepositoryUseCaseError> {
        log::info!("archive repository: {}", full_name);
        let repository = self
            .github_service
            .get_repository(full_name)
            .await
            .map_err(|_| GetGitHubRepositoryUseCaseError::Error);

        match repository {
            Ok(Some(repository)) => {
                log::debug!("repository: {:#?}", repository);
                let autolink_references = self
                    .github_service
                    .get_autolink_references(full_name)
                    .await
                    .map_err(|_| GetGitHubRepositoryUseCaseError::Error)?
                    .iter()
                    .map(|item| item.clone().into())
                    .collect::<Vec<AutolinkReference>>();
                let autolink_references = if autolink_references.is_empty() {
                    None
                } else {
                    Some(autolink_references)
                };
                Ok(Some(GitHubRepositorySpec {
                    full_name: full_name.into(),
                    repository: Some(repository),
                    autolink_references,
                    permissions: None,
                }))
            }
            Ok(None) => Ok(None),
            Err(_) => Err(GetGitHubRepositoryUseCaseError::Error),
        }
    }
}

pub enum GetGitHubRepositoryUseCaseError {
    Error,
}

impl From<AutolinkReferenceResponse> for AutolinkReference {
    fn from(value: AutolinkReferenceResponse) -> Self {
        Self {
            key_prefix: value.key_prefix,
            url_template: value.url_template,
            is_alphanumeric: value.is_alphanumeric,
        }
    }
}
