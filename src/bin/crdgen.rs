use github_operator::domain::model::autolink_reference::AutolinkReference;
use github_operator::domain::model::repository::Repository;
use kube::CustomResourceExt;

fn main() {
    let crds = vec![
        serde_yaml::to_string(&Repository::crd()).unwrap(),
        serde_yaml::to_string(&AutolinkReference::crd()).unwrap(),
    ];
    print!("{}", crds.join("\n---\n"))
}
