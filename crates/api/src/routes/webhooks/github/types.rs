use serde::{Deserialize, Serialize};
use serde_json::Value;
use sonm_result::{Result, create_error};

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub login: String,
    pub id: u32,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositorySecurityAndAnalysisStatus {
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositorySecurityAndAnalysis {
    pub advanced_security: GithubRepositorySecurityAndAnalysisStatus,
    pub secret_scanning: GithubRepositorySecurityAndAnalysisStatus,
    pub secret_scanning_push_protection: GithubRepositorySecurityAndAnalysisStatus,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositoryLicense {
    pub key: Option<String>,
    pub name: Option<String>,
    pub spdx_id: Option<String>,
    pub url: Option<String>,
    pub node_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositoryCodeOfConduct {
    pub key: String,
    pub name: String,
    pub url: String,
    pub body: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositoryPermissions {
    pub admin: Option<bool>,
    pub maintain: Option<bool>,
    pub push: Option<bool>,
    pub triage: Option<bool>,
    pub pull: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepository {
    pub id: u32,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub owner: GithubUser,
    pub private: bool,
    pub html_url: String,
    pub description: Option<String>,
    pub fork: bool,
    pub url: String,
    pub archive_url: String,
    pub assignees_url: String,
    pub blobs_url: String,
    pub branches_url: String,
    pub collaborators_url: String,
    pub comments_url: String,
    pub commits_url: String,
    pub compare_url: String,
    pub contents_url: String,
    pub contributors_url: String,
    pub deployments_url: String,
    pub downloads_url: String,
    pub events_url: String,
    pub forks_url: String,
    pub git_commits_url: String,
    pub git_refs_url: String,
    pub git_tags_url: String,
    pub git_url: Option<String>,
    pub issue_comment_url: String,
    pub issue_events_url: String,
    pub issues_url: String,
    pub keys_url: String,
    pub labels_url: String,
    pub languages_url: String,
    pub merges_url: String,
    pub milestones_url: String,
    pub notifications_url: String,
    pub pulls_url: String,
    pub releases_url: String,
    pub ssh_url: String,
    pub stargazers_url: String,
    pub statuses_url: String,
    pub subscribers_url: String,
    pub subscription_url: String,
    pub tags_url: String,
    pub teams_url: String,
    pub trees_url: String,
    pub clone_url: Option<String>,
    pub mirror_url: Option<String>,
    pub hooks_url: String,
    pub svn_url: Option<String>,
    pub homepage: Option<String>,
    pub language: Option<String>,
    pub forks_count: Option<u32>,
    pub stargazers_count: Option<u32>,
    pub watchers_count: Option<u32>,
    pub size: Option<u64>,
    pub default_branch: Option<String>,
    pub open_issues_count: Option<u32>,
    pub is_template: Option<bool>,
    pub topics: Option<Vec<String>>,
    pub has_issues: Option<bool>,
    pub has_projects: Option<bool>,
    pub has_wiki: Option<bool>,
    pub has_pages: Option<bool>,
    pub has_downloads: Option<bool>,
    pub has_discussions: Option<bool>,
    pub archived: Option<bool>,
    pub disabled: Option<bool>,
    pub visibility: Option<String>,
    pub pushed_at: Option<Value>,
    pub created_at: Value,
    pub updated_at: Option<Value>,
    pub permissions: Option<GithubRepositoryPermissions>,
    pub role_name: Option<String>,
    pub temp_clone_token: Option<String>,
    pub delete_branch_on_merge: Option<bool>,
    pub subscribers_count: Option<u32>,
    pub network_count: Option<u32>,
    pub code_of_conduct: Option<GithubRepositoryCodeOfConduct>,
    pub license: Option<GithubRepositoryLicense>,
    pub forks: Option<u32>,
    pub open_issues: Option<u32>,
    pub watchers: Option<u32>,
    pub allow_forking: Option<bool>,
    pub web_commit_signoff_required: Option<bool>,
    pub security_and_analysis: Option<GithubRepositorySecurityAndAnalysis>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CommitAuthor {
    pub date: Option<String>,
    pub email: Option<String>,
    pub name: String,
    pub username: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubCommit {
    pub added: Option<Vec<String>>,
    pub author: CommitAuthor,
    pub committer: CommitAuthor,
    pub distinct: bool,
    pub id: String,
    pub message: String,
    pub modified: Option<Vec<String>>,
    pub removed: Option<Vec<String>>,
    pub timestamp: String,
    pub tree_id: String,
    pub url: String,
}
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubReactions {
    #[serde(rename = "+1")]
    pub plus_one: u32,
    #[serde(rename = "-1")]
    pub minus_one: u32,
    pub confused: u32,
    pub eyes: u32,
    pub heart: u32,
    pub hooray: u32,
    pub laugh: u32,
    pub rocket: u32,
    pub total_count: u32,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubComment {
    pub author_association: String,
    pub body: String,
    pub commit_id: Option<String>,
    pub created_at: Value,
    pub html_url: String,
    pub id: u32,
    pub line: Option<u32>,
    pub node_id: String,
    pub path: Option<String>,
    pub position: Option<u32>,
    pub reactions: Option<GithubReactions>,
    pub updated_at: Value,
    pub user: GithubUser,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubDiscussionComment {
    #[serde(flatten)]
    pub comment: GithubComment,
    pub child_comment_count: u32,
    pub parent_id: Option<u32>,
    pub repository_url: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubDiscussionCategory {
    pub id: u32,
    pub node_id: String,
    pub repository_id: u32,
    pub emoji: String,
    pub name: String,
    pub description: String,
    pub created_at: Value,
    pub updated_at: Value,
    pub slug: String,
    pub is_answerable: bool,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubDiscussion {
    pub repository_url: String,
    pub category: GithubDiscussionCategory,
    pub answer_html_url: Option<String>,
    pub answer_chosen_at: Value, // ??
    pub answer_chosen_by: Value, // ??
    pub html_url: String,
    pub id: u32,
    pub node_id: String,
    pub number: u32,
    pub title: String,
    pub user: GithubUser,
    pub state: String,
    pub locked: bool,
    pub comments: u32,
    pub created_at: Value,
    pub updated_at: Value,
    pub author_association: String,
    pub active_lock_reason: Option<String>,
    pub body: String,
    pub reactions: GithubReactions,
    pub timeline_url: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "action")]
#[allow(clippy::large_enum_variant)]
pub enum GithubDiscussionEvent {
    Created {
        discussion: GithubDiscussion,
    },
    Answered {
        discussion: GithubDiscussion,
        answer: GithubDiscussionComment,
    },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "action")]
#[allow(clippy::large_enum_variant)]
pub enum DiscussionCommentEvent {
    Created {
        comment: GithubDiscussionComment,
        discussion: GithubDiscussion,
    },
    Deleted {},
    Edited {},
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubMilestone {
    pub url: String,
    pub html_url: String,
    pub labels_url: String,
    pub id: u32,
    pub node_id: String,
    pub number: u32,
    pub state: String,
    pub title: String,
    pub description: String,
    pub creator: Option<GithubUser>,
    pub open_issues: u32,
    pub closed_issues: u32,
    pub created_at: Value,
    pub updated_at: Value,
    pub closed_at: Option<Value>,
    pub due_on: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubLinkedPullRequest {
    pub merged_at: Option<Value>,
    pub diff_url: Option<String>,
    pub html_url: Option<String>,
    pub patch_url: Option<String>,
    pub url: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubAppPermissions {
    pub issues: Option<String>,
    pub checks: Option<String>,
    pub metadata: Option<String>,
    pub contents: Option<String>,
    pub deployments: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubApp {
    pub id: u32,
    pub slug: String,
    pub node_is: String,
    pub owner: Option<GithubUser>,
    pub name: String,
    pub description: Option<String>,
    pub external_url: String,
    pub html_url: String,
    pub created_at: Value,
    pub updated_at: Value,
    pub permissions: GithubAppPermissions,
    pub events: Vec<String>,
    pub installations_count: Option<u32>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub webhook_secret: Option<String>,
    pub pem: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubIssue {
    pub id: u32,
    pub node_id: String,
    pub url: String,
    pub repository_url: String,
    pub labels_url: String,
    pub comments_url: String,
    pub events_url: String,
    pub html_url: String,
    pub number: u32,
    pub state: String,
    pub state_reason: Option<String>,
    pub user: Option<GithubUser>,
    pub labels: Vec<String>,
    pub assignee: Option<GithubUser>,
    pub assignees: Vec<GithubUser>,
    pub milestone: Option<GithubMilestone>,
    pub locked: bool,
    pub active_lock_reason: Option<String>,
    pub comments: u32,
    pub pull_request: Option<GithubLinkedPullRequest>,
    pub closed_at: Option<Value>,
    pub created_at: Value,
    pub updated_at: Value,
    pub draft: Option<bool>,
    pub closed_by: Option<GithubUser>,
    pub body: Option<String>,
    pub timeline_url: Option<String>,
    pub repository: Option<GithubRepository>,
    pub performed_via_github_app: Option<GithubApp>,
    pub author_association: String,
    pub reactions: Option<GithubReactions>,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "action")]
#[allow(clippy::large_enum_variant)]
pub enum IssueCommentEvent {
    Created {
        comment: GithubComment,
        issue: GithubIssue,
    },
    Deleted {},
    Edited {},
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "action")]
#[allow(clippy::large_enum_variant)]
pub enum IssuesEvent {
    Assigned {},
    Closed { issue: GithubIssue },
    Deleted {},
    Demilestoned {},
    Edited {},
    Labeled {},
    Locked {},
    Milestoned {},
    Opened { issue: GithubIssue },
    Pinned {},
    Reopened { issue: GithubIssue },
    Transferred {},
    Unassigned {},
    Unlabeled {},
    Unlocked {},
    Unpinned {},
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StarEvent {
    pub starred_at: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PushEvent {
    pub after: String,
    pub base_ref: Option<String>,
    pub before: String,
    pub commits: Vec<GithubCommit>,
    pub compare: String,
    pub created: bool,
    pub deleted: bool,
    pub forced: bool,
    pub head_commit: Option<GithubCommit>,
    pub pusher: CommitAuthor,
    pub r#ref: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitCommentEvent {
    pub comment: GithubComment,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateEvent {
    pub master_branch: String,
    pub pusher_type: String,
    pub r#ref: String,
    pub ref_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteEvent {
    pub pusher_type: String,
    pub r#ref: String,
    pub ref_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForkEvent {
    pub forkee: GithubRepository,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubTeam {
    pub id: u32,
    pub node_id: String,
    pub url: String,
    pub hmtl_url: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub privacy: String,
    pub permission: String,
    pub members_url: String,
    pub repositories_url: String,
    pub parent: Option<Box<GithubTeam>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubHead {
    pub label: String,
    pub r#ref: String,
    pub repo: GithubRepository,
    pub sha: String,
    pub user: Option<GithubUser>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubHref {
    pub href: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubLinks {
    #[serde(rename = "self")]
    pub _self: GithubHref,
    pub html: GithubHref,
    pub comments: GithubHref,
    pub commits: GithubHref,
    pub statuses: GithubHref,
    pub issue: GithubHref,
    pub review_comments: GithubHref,
    pub review_comment: GithubHref,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubAutoMerge {
    pub enabled_by: GithubUser,
    pub merge_method: String,
    pub commit_title: String,
    pub commit_message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubPullRequest {
    pub url: String,
    pub id: u32,
    pub node_id: String,
    pub html_url: String,
    pub diff_url: String,
    pub patch_url: String,
    pub issue_url: String,
    pub number: u32,
    pub state: String,
    pub locked: bool,
    pub title: String,
    pub user: GithubUser,
    pub body: Option<String>,
    pub created_at: Value,
    pub updated_at: Value,
    pub closed_at: Option<Value>,
    pub merged_at: Option<Value>,
    pub merge_commit_sha: Option<String>,
    pub assignee: Option<GithubUser>,
    pub assignees: Vec<GithubUser>,
    pub requested_reviewers: Vec<GithubUser>,
    pub requested_teams: Vec<GithubTeam>,
    pub head: GithubHead,
    pub base: GithubHead,
    pub _links: GithubLinks,
    pub author_association: String,
    pub auto_merge: Option<GithubAutoMerge>,
    pub draft: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum PullRequestEvent {
    Assigned {},
    AutoMergeDisabled {},
    AutoMergeEnabled {},
    Closed {
        number: u32,
        pull_request: GithubPullRequest,
    },
    ConvertedToDraft {},
    Demilestoned {},
    Dequeued {},
    Edited {},
    Enqueued {},
    Labeled {},
    Locked {},
    Milestoned {},
    Opened {
        number: u32,
        pull_request: GithubPullRequest,
    },
    ReadyForReview {},
    Reopened {
        number: u32,
        pull_request: GithubPullRequest,
    },
    ReviewRequestRemoved {},
    ReviewRequest {},
    Synchronized {},
    Unassigned {},
    Unlabeled {},
    Unlocked {},
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum BaseEvent {
    #[allow(dead_code)]
    Star(StarEvent),
    Ping,
    Push(PushEvent),
    CommitComment(CommitCommentEvent),
    Create(CreateEvent),
    Delete(DeleteEvent),
    Discussion(GithubDiscussionEvent),
    DiscussionComment(DiscussionCommentEvent),
    Fork(ForkEvent),
    IssueComment(IssueCommentEvent),
    Issues(IssuesEvent),
    PullRequest(PullRequestEvent),
}

#[derive(Serialize, Deserialize, Debug)]
struct _Event {
    action: Option<String>,
    sender: GithubUser,
    repository: GithubRepository,
}

#[derive(Debug)]
pub struct Event {
    pub event: BaseEvent,
    pub action: Option<String>,
    pub sender: GithubUser,
    pub repository: GithubRepository,
}

fn safe_from_str<T: for<'de> Deserialize<'de>>(data: &str) -> Result<T> {
    match serde_json::from_str(data) {
        Ok(output) => Ok(output),
        Err(err) => {
            sonm_config::capture_internal_error!(err);
            Err(create_error!(InvalidOperation))
        }
    }
}

// Because github gives the event name in the header, this requires us to manually parse the enum, i did try to manually edit the json to add the tag into it,
// however it seems serde messes up with #[serde(flattern)] and enum handling.
// If someone finds a better solution to this please make a PR.

pub fn convert_event(data: &str, event_name: &str) -> Result<Event> {
    let event = safe_from_str(data)?;

    let base_event = match event_name {
        "star" => BaseEvent::Star(safe_from_str(data)?),
        "ping" => BaseEvent::Ping,
        "push" => BaseEvent::Push(safe_from_str(data)?),
        "commit_comment" => BaseEvent::CommitComment(safe_from_str(data)?),
        "create" => BaseEvent::Create(safe_from_str(data)?),
        "delete" => BaseEvent::Delete(safe_from_str(data)?),
        "discussion" => BaseEvent::Discussion(safe_from_str(data)?),
        "discussion_comment" => BaseEvent::DiscussionComment(safe_from_str(data)?),
        "fork" => BaseEvent::Fork(safe_from_str(data)?),
        "issue_comment" => BaseEvent::IssueComment(safe_from_str(data)?),
        "issues" => BaseEvent::Issues(safe_from_str(data)?),
        "pull_request" => BaseEvent::PullRequest(safe_from_str(data)?),
        _ => return Err(create_error!(InvalidOperation)),
    };

    let _Event {
        action,
        sender,
        repository,
    } = event;

    Ok(Event {
        action,
        sender,
        repository,
        event: base_event,
    })
}
