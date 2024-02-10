use crate::domain::service::github_service::GitHubService;
use crate::domain::spec::repository_spec::RepositorySpec;
use crate::ControllerError;

pub struct ArchiveRepositoryUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl ArchiveRepositoryUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(&self, spec: &RepositorySpec) -> Result<(), ControllerError> {
        log::info!("archive repository: {}", &spec.full_name);
        self.github_service
            .archive_repository(&spec.full_name)
            .await
    }
}
