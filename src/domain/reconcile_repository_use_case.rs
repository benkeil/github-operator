use differ_from_spec::DifferFromSpec;
use k8s_openapi::api::core::v1::ObjectReference;
use kube::runtime::events::{Event, EventType, Recorder};
use kube::Resource;

use crate::domain::model::repository::{Repository, RepositoryResponse, RepositorySpec};
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
        github_repository: &Repository,
        recorder: Recorder,
    ) -> Result<(), ControllerError> {
        log::info!("reconcile: {}", &github_repository.spec.full_name);

        let reference = github_repository.object_ref(&());

        // TODO double clone
        // enable additional settings if necessary
        let spec_repository: RepositoryResponse =
            github_repository.spec.clone().auto_configure().into();

        let repository = self
            .get_or_create_repository(
                &github_repository.spec.full_name,
                &spec_repository,
                &reference,
                &recorder,
            )
            .await?;
        log::debug!("repository: {:#?}", repository);

        if repository.differ_from_spec(&spec_repository) {
            log::info!("repository needs to be updated");
            self.github_service
                .update_repository(&github_repository.spec.full_name, &spec_repository)
                .await?;
            self.publish_updated_event(&recorder, &reference).await?;
        }

        Ok(())
    }

    async fn get_or_create_repository(
        &self,
        full_name: &str,
        repository_spec: &RepositoryResponse,
        reference: &ObjectReference,
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
                self.publish_created_event(recorder, reference).await?;
                Ok(repository)
            }
            Err(e) => Err(e),
        }
    }

    async fn publish_created_event(
        &self,
        recorder: &Recorder,
        reference: &ObjectReference,
    ) -> Result<(), ControllerError> {
        recorder
            .publish(
                &Event {
                    action: "repository-created".into(),
                    reason: "Reconciling".into(),
                    note: Some("GitHub repository created".into()),
                    type_: EventType::Normal,
                    secondary: None,
                },
                reference,
            )
            .await
            .map_err(ControllerError::KubeError)
    }

    async fn publish_updated_event(
        &self,
        recorder: &Recorder,
        reference: &ObjectReference,
    ) -> Result<(), ControllerError> {
        recorder
            .publish(
                &Event {
                    action: "repository-updated".into(),
                    reason: "Reconciling".into(),
                    note: Some("GitHub repository updated".into()),
                    type_: EventType::Normal,
                    secondary: None,
                },
                reference,
            )
            .await
            .map_err(ControllerError::KubeError)
    }
}
