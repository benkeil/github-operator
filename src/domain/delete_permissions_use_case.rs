use kube::runtime::events::{Event, EventType, Recorder};

use crate::domain::model::permission::RepositoryPermissionSpec;
use crate::domain::service::github_service::GitHubService;
use crate::ControllerError;

pub struct DeletePermissionUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl DeletePermissionUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        spec: &RepositoryPermissionSpec,
        recorder: Recorder,
    ) -> Result<(), ControllerError> {
        log::info!("delete: {}", &spec.full_name);

        self.github_service
            .delete_team_permission(&spec.full_name, &spec.full_team_name)
            .await?;
        self.publish_deleted_event(&recorder).await
    }

    async fn publish_deleted_event(&self, recorder: &Recorder) -> Result<(), ControllerError> {
        recorder
            .publish(Event {
                action: "permission-deleted".into(),
                reason: "Reconciling".into(),
                note: Some("Permission deleted".into()),
                type_: EventType::Normal,
                secondary: None,
            })
            .await
            .map_err(ControllerError::KubeError)
    }
}
