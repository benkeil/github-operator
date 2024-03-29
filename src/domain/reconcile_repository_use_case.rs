use differ_from_spec::DifferFromSpec;
use kube::runtime::events::{Event, EventType, Recorder};

use crate::domain::model::repository::{RepositoryResponse, RepositorySpec};
use crate::domain::model::AutoConfigureSpec;
use crate::domain::service::github_service::GitHubService;
use crate::ControllerError;

pub struct ReconcileRepositoryUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl ReconcileRepositoryUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        spec: &RepositorySpec,
        recorder: Recorder,
    ) -> Result<(), ControllerError> {
        log::info!("reconcile: {}", &spec.full_name);

        // TODO double clone
        // enable additional settings if necessary
        let spec_repository: RepositoryResponse = spec.clone().auto_configure().into();

        let repository = self
            .get_or_create_repository(&spec.full_name, &spec_repository, &recorder)
            .await?;
        log::debug!("repository: {:#?}", repository);

        if repository.differ_from_spec(&spec_repository) {
            log::info!("repository needs to be updated");
            self.github_service
                .update_repository(&spec.full_name, &spec_repository)
                .await?;
            self.publish_updated_event(&recorder).await?;
        }

        Ok(())
    }

    async fn get_or_create_repository(
        &self,
        full_name: &str,
        repository_spec: &RepositoryResponse,
        recorder: &Recorder,
    ) -> Result<RepositoryResponse, ControllerError> {
        let repository = self.github_service.get_repository(full_name).await;
        match repository {
            Ok(Some(repository)) => Ok(repository),
            Ok(None) => {
                let repository = self
                    .github_service
                    .create_repository(full_name, repository_spec)
                    .await?;
                self.publish_created_event(recorder).await?;
                Ok(repository)
            }
            Err(e) => Err(e),
        }
    }

    async fn publish_created_event(&self, recorder: &Recorder) -> Result<(), ControllerError> {
        recorder
            .publish(Event {
                action: "repository-created".into(),
                reason: "Reconciling".into(),
                note: Some("GitHub repository created".into()),
                type_: EventType::Normal,
                secondary: None,
            })
            .await
            .map_err(ControllerError::KubeError)
    }

    async fn publish_updated_event(&self, recorder: &Recorder) -> Result<(), ControllerError> {
        recorder
            .publish(Event {
                action: "repository-updated".into(),
                reason: "Reconciling".into(),
                note: Some("GitHub repository updated".into()),
                type_: EventType::Normal,
                secondary: None,
            })
            .await
            .map_err(ControllerError::KubeError)
    }
}
