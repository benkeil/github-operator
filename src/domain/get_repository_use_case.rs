use crate::domain::model::RepositoryFullView;
use crate::domain::service::github_service::GitHubService;
use crate::ControllerError;

pub struct GetRepositoryUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl GetRepositoryUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        full_name: &str,
    ) -> Result<Option<RepositoryFullView>, ControllerError> {
        log::info!("get repository: {}", full_name);
        let repository = self.github_service.get_repository(full_name).await;

        match repository {
            Ok(Some(repository)) => {
                log::debug!("repository: {:#?}", repository);
                let autolink_references = self
                    .github_service
                    .get_autolink_references(full_name)
                    .await?;
                let autolink_references = if autolink_references.is_empty() {
                    None
                } else {
                    Some(autolink_references)
                };
                Ok(Some(RepositoryFullView {
                    full_name: full_name.into(),
                    repository: Some(repository),
                    autolink_references,
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
