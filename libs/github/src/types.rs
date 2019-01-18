use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Json,
    Form,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InsecureSsl {
    #[serde(rename = "0")]
    ValidateCerts,
    #[serde(rename = "1")]
    InsecureSkipValidation,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HookConfig {
    pub url: String,
    pub content_type: ContentType,
    pub secret: Option<String>,
    pub insecure_ssl: InsecureSsl,
}

impl std::cmp::PartialEq for HookConfig {
    fn eq(&self, other: &HookConfig) -> bool {
        self.url == other.url
            && self.content_type == other.content_type
            && self.insecure_ssl == other.insecure_ssl
    }
}

impl std::cmp::Eq for HookConfig {}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HookDetails {
    pub name: String,
    pub events: HashSet<EventKind>,
    pub active: bool,
    pub config: HookConfig,
}

#[derive(Debug, serde::Deserialize)]
pub struct Hook {
    pub id: usize,
    #[serde(flatten)]
    pub details: HookDetails,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    CheckRun,
    CheckSuite,
    CommitComment,
    Create,
    Delete,
    Deployment,
    DeploymentStatus,
    Fork,
    GithubAppAuthorization,
    Gollum,
    Installation,
    InstallationRepositories,
    IssueComment,
    Issues,
    Label,
    MarketplacePurchase,
    Member,
    Membership,
    Milestone,
    OrgBlock,
    Organization,
    PageBuild,
    Project,
    ProjectCard,
    ProjectColumn,
    Public,
    PullRequest,
    PullRequestReview,
    PullRequestReviewComment,
    Push,
    Release,
    Repository,
    RepositoryImport,
    RepositoryVulnerabilityAlert,
    SecurityAdvisory,
    Status,
    Team,
    TeamAdd,
    Watch,
    #[serde(rename = "*")]
    Wildcard,
}
