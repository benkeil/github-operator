use github_operator::domain::spec::autolink_reference_spec::AutolinkReference;
use github_operator::domain::spec::repository_spec::Repository;
use kube::CustomResourceExt;

fn main() {
    let crds = vec![
        serde_yaml::to_string(&Repository::crd()).unwrap(),
        serde_yaml::to_string(&AutolinkReference::crd()).unwrap(),
    ];
    print!("{}", crds.join("\n---\n"))
}
