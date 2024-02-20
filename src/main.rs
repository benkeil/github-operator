use std::net::SocketAddr;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::TryFutureExt;
use kube::api::ListParams;
use kube::{Api, Client};
use prometheus::{Encoder, Registry, TextEncoder};
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinSet;
use tracing::event;

use github_operator::{init_meter, init_tracing, ControllerError};

use crate::adapter::http_github_service::HttpGithubService;
use crate::controller::autolink_reference_controller::{self, AutolinkReferenceControllerContext};
use crate::controller::permission_controller;
use crate::controller::permission_controller::PermissionControllerContext;
use crate::controller::repository_controller::{self, RepositoryControllerContext};
use crate::domain::archive_repository_use_case::ArchiveRepositoryUseCase;
use crate::domain::delete_autolink_reference_use_case::DeleteAutolinkReferenceUseCase;
use crate::domain::delete_permissions_use_case::DeletePermissionUseCase;
use crate::domain::model::autolink_reference::AutolinkReference;
use crate::domain::model::permission::RepositoryPermission;
use crate::domain::model::repository::Repository;
use crate::domain::reconcile_autolink_reference_use_case::ReconcileAutolinkReferenceUseCase;
use crate::domain::reconcile_permissions_use_case::ReconcilePermissionUseCase;
use crate::domain::reconcile_repository_use_case::ReconcileRepositoryUseCase;

mod adapter;
mod controller;
mod domain;
mod extensions;

#[tokio::main]
async fn main() -> Result<(), ControllerError> {
    init_tracing()?;
    event!(tracing::Level::INFO, "starting controllers...");

    let registry = Registry::new();
    let meter = init_meter(&registry)?;

    let client = Client::try_default()
        .await
        .map_err(ControllerError::KubeError)?;

    // the kubernetes API for our CRD
    let repository_api = Api::<Repository>::all(client.clone());
    let autolink_reference_api = Api::<AutolinkReference>::all(client.clone());
    let permission_api = Api::<RepositoryPermission>::all(client.clone());

    // check if the CRD is installed, or else throw an error
    repository_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;
    autolink_reference_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;
    permission_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;

    // dependencies
    let github_service = HttpGithubService::from_env();

    let mut tasks = JoinSet::new();

    // start server to expose metrics
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(HttpState {
            registry: registry.clone(),
        });
    let addr = SocketAddr::from(([127, 0, 0, 1], 9100));
    let server = axum_server::bind(addr)
        .serve(app.into_make_service())
        .map_err(ControllerError::IoError);
    let http_handle = tokio::spawn(server);

    // add repository controller
    tasks.spawn(repository_controller::run(RepositoryControllerContext {
        client: client.clone(),
        meter,
        repository_api,
        reconcile_use_case: ReconcileRepositoryUseCase::new(Box::new(github_service.clone())),
        archive_use_case: ArchiveRepositoryUseCase::new(Box::new(github_service.clone())),
    }));

    // add autolink reference controller
    tasks.spawn(autolink_reference_controller::run(
        AutolinkReferenceControllerContext {
            client: client.clone(),
            autolink_reference_api,
            reconcile_use_case: ReconcileAutolinkReferenceUseCase::new(Box::new(
                github_service.clone(),
            )),
            delete_use_case: DeleteAutolinkReferenceUseCase::new(Box::new(github_service.clone())),
        },
    ));

    // add permission controller
    tasks.spawn(permission_controller::run(PermissionControllerContext {
        client: client.clone(),
        permission_api,
        reconcile_use_case: ReconcilePermissionUseCase::new(Box::new(github_service.clone())),
        delete_use_case: DeletePermissionUseCase::new(Box::new(github_service.clone())),
    }));

    while let Some(res) = tasks.join_next().await {
        if let Err(e) = res {
            event!(tracing::Level::ERROR, "error: {:?}", e);
        }
    }

    // Listen for SIGINT signal for graceful shutdown
    let mut stream = signal(SignalKind::interrupt()).unwrap();
    tokio::spawn(async move {
        stream.recv().await;
        http_handle.abort();
    });

    Ok(())
}

#[derive(Clone)]
struct HttpState {
    pub registry: Registry,
}

async fn metrics_handler(State(HttpState { registry }): State<HttpState>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut result = Vec::new();
    encoder
        .encode(&metric_families, &mut result)
        .map_err(ControllerError::PrometheusError)
        .expect("Couldn't encode metrics");
    result
}
