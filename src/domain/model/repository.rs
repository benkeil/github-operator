#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Repository {
    pub full_name: String,
    pub security_and_analysis: SecurityAndAnalysis,
}

#[derive(Debug, Hash, Eq, PartialEq)]
#[non_exhaustive]
pub struct SecurityAndAnalysis {
    pub secret_scanning: Status,
    pub secret_scanning_push_protection: Status,
    pub dependabot_security_updates: Status,
    pub secret_scanning_validity_checks: Status,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Status {
    Enabled,
    Disabled,
}
