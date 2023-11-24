use crate::controller::State;

mod controller;
mod domain;
mod extensions;
mod adapter;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!("start controller");
    let state = State::default();
    if let Err(e) = controller::run(state).await {
        log::error!("{}", e);
    }
}
