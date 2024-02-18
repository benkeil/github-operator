pub mod autolink_reference_controller;
pub mod permission_controller;
pub mod repository_controller;

pub fn finalizer_name(controller_name: &str) -> String {
    format!("{}.github.platform.benkeil.de/finalizer", controller_name)
}
