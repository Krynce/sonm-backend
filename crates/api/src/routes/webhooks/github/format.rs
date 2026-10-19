use sonm_models::v0::SendableEmbed;

use super::types::*;

const GREEN: &str = "#0f890f";
const RED: &str = "#e73e3e";
const GREY: &str = "#202224";
const BLUE: &str = "#279adc";
const ORANGE: &str = "#d76a34";
const LIGHT_ORANGE: &str = "#d9916d";
// for future use
// const WHITE: &str = "#c3e1c3";

/// Cut `text` down to at most `length` characters, appending `...` when something was cut.
fn shorten_text(text: &str, length: usize) -> String {
    if text.chars().count() > length {
        format!(
            "{}...",
            text.chars()
                .take(length.saturating_sub(3))
                .collect::<String>()
        )
    } else {
        text.to_string()
    }
}

/// Same as [`shorten_text`], but keeps only the first line.
fn shorten_single_line_text(text: &str, length: usize) -> String {
    match text.split_once('\n') {
        Some((first, _)) => {
            let shortened = shorten_text(first, length);

            if shortened.ends_with("...") {
                shortened
            } else {
                format!("{shortened}...")
            }
        }
        None => shorten_text(text, length),
    }
}

/// First 8 characters of a commit SHA, or the whole thing if it is shorter.
fn short_sha(id: &str) -> String {
    id.chars().take(8).collect()
}

/// Render a GitHub event as an embed, or `None` for events we stay quiet about
pub fn event_embed(event: Event) -> Option<SendableEmbed> {
    let sendable_embed = match event.event {
        BaseEvent::Star(_) => {
            if event.action.as_deref() != Some("created") {
                return None;
            };

            SendableEmbed {
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] New star added]({})",
                    event.repository.full_name, event.repository.html_url
                )),
                colour: Some(GREY.to_string()),
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                ..Default::default()
            }
        }
        BaseEvent::Push(PushEvent {
            after,
            commits,
            compare,
            forced,
            r#ref,
            ..
        }) => {
            let branch = r#ref.split('/').skip(2).collect::<Vec<_>>().join("/");

            if forced {
                let description = format!(
                    "#### [{}] Branch {} was force-pushed to {}\n[compare changes]({})",
                    event.repository.full_name,
                    branch,
                    &short_sha(&after),
                    compare
                );

                SendableEmbed {
                    icon_url: Some(event.sender.avatar_url),
                    title: Some(event.sender.login),
                    description: Some(description),
                    url: Some(event.sender.html_url),
                    colour: Some(RED.to_string()),
                    ..Default::default()
                }
            } else {
                let title = format!(
                    "[[{}:{}] {} new commit]({})",
                    event.repository.full_name,
                    branch,
                    commits.len(),
                    compare
                );
                let commit_description = shorten_text(
                    &commits
                        .into_iter()
                        .map(|commit| {
                            format!(
                                "[`{}`]({}) {} - {}",
                                &short_sha(&commit.id),
                                commit.url,
                                shorten_single_line_text(&commit.message, 50),
                                commit.author.name
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("\n"),
                    1000,
                );

                SendableEmbed {
                    title: Some(event.sender.login),
                    description: Some(format!("#### {title}\n{commit_description}")),
                    colour: Some(BLUE.to_string()),
                    icon_url: Some(event.sender.avatar_url),
                    url: Some(event.sender.html_url),
                    ..Default::default()
                }
            }
        }
        BaseEvent::CommitComment(CommitCommentEvent { comment }) => {
            let commit_id = match comment.commit_id {
                Some(id) => short_sha(&id),
                None => "".to_string(),
            };

            SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] New comment on commit `{}`]({})\n{}",
                    event.repository.full_name,
                    commit_id,
                    comment.html_url,
                    shorten_text(&comment.body, 450)
                )),
                colour: Some(GREY.to_string()),
                ..Default::default()
            }
        }
        BaseEvent::Ping => return None,
        BaseEvent::Create(CreateEvent {
            r#ref, ref_type, ..
        }) => SendableEmbed {
            icon_url: Some(event.sender.avatar_url),
            url: Some(event.sender.html_url),
            title: Some(event.sender.login),
            description: Some(format!(
                "#### [{}] New {} created: {}",
                event.repository.full_name, ref_type, r#ref
            )),
            colour: Some(GREY.to_string()),
            ..Default::default()
        },
        BaseEvent::Delete(DeleteEvent {
            r#ref, ref_type, ..
        }) => SendableEmbed {
            icon_url: Some(event.sender.avatar_url),
            url: Some(event.sender.html_url),
            title: Some(event.sender.login),
            description: Some(format!(
                "#### [{}] {} deleted: {}",
                event.repository.full_name, ref_type, r#ref
            )),
            colour: Some(GREY.to_string()),
            ..Default::default()
        },
        BaseEvent::Discussion(discussion_event) => match discussion_event {
            GithubDiscussionEvent::Created { discussion } => SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] New discussion #{}: {}]({})\n{}",
                    event.repository.full_name,
                    discussion.number,
                    discussion.title,
                    discussion.html_url,
                    shorten_text(&discussion.body, 450)
                )),
                colour: Some(LIGHT_ORANGE.to_string()),
                ..Default::default()
            },
            GithubDiscussionEvent::Answered { discussion, answer } => SendableEmbed {
                icon_url: Some(answer.comment.user.avatar_url),
                url: Some(answer.comment.user.html_url),
                title: Some(answer.comment.user.login),
                description: Some(format!(
                    "#### [[{}] Discussion #{} marked answered: {}]({})\n{}",
                    event.repository.full_name,
                    discussion.number,
                    discussion.title,
                    answer.comment.html_url,
                    shorten_text(&answer.comment.body, 450)
                )),
                colour: Some(LIGHT_ORANGE.to_string()),
                ..Default::default()
            },
        },
        BaseEvent::DiscussionComment(comment_event) => match comment_event {
            DiscussionCommentEvent::Created {
                comment,
                discussion,
            } => SendableEmbed {
                icon_url: Some(comment.comment.user.avatar_url),
                url: Some(comment.comment.user.html_url),
                title: Some(comment.comment.user.login),
                description: Some(format!(
                    "#### [[{}] New comment on discussion #{}: {}]({})\n{}",
                    event.repository.full_name,
                    discussion.number,
                    discussion.title,
                    comment.comment.html_url,
                    shorten_text(&comment.comment.body, 450)
                )),
                colour: Some(LIGHT_ORANGE.to_string()),
                ..Default::default()
            },
            _ => return None,
        },
        BaseEvent::Fork(ForkEvent { forkee }) => SendableEmbed {
            icon_url: Some(event.sender.avatar_url),
            url: Some(event.sender.html_url),
            title: Some(event.sender.login),
            description: Some(format!(
                "#### [[{}] Fork created: {}]({})",
                event.repository.full_name, forkee.full_name, forkee.html_url
            )),
            colour: Some(GREY.to_string()),
            ..Default::default()
        },
        BaseEvent::Issues(issue_event) => match issue_event {
            IssuesEvent::Closed { issue } => SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] Issue Closed #{}: {}]({})",
                    event.repository.full_name, issue.number, issue.title, issue.html_url
                )),
                colour: Some(GREY.to_string()),
                ..Default::default()
            },
            IssuesEvent::Opened { issue } => SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] Issue Opened #{}: {}]({})\n{}",
                    event.repository.full_name,
                    issue.number,
                    issue.title,
                    issue.html_url,
                    shorten_text(&issue.body.unwrap_or_default(), 450)
                )),
                colour: Some(ORANGE.to_string()),
                ..Default::default()
            },
            IssuesEvent::Reopened { issue } => SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] Issue Reopened #{}: {}]({})",
                    event.repository.full_name, issue.number, issue.html_url, issue.title
                )),
                colour: Some(GREEN.to_string()),
                ..Default::default()
            },
            _ => return None,
        },
        BaseEvent::IssueComment(comment_event) => match comment_event {
            IssueCommentEvent::Created { comment, issue } => SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] New comment on issue #{}: {}]({})\n{}",
                    event.repository.full_name,
                    issue.number,
                    issue.title,
                    comment.html_url,
                    shorten_text(&comment.body, 450)
                )),
                colour: Some(LIGHT_ORANGE.to_string()),
                ..Default::default()
            },
            _ => return None,
        },
        BaseEvent::PullRequest(pull_request_event) => match pull_request_event {
            PullRequestEvent::Closed {
                number,
                pull_request,
            } => SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] Pull Request Closed #{}: {}]({})",
                    event.repository.full_name, number, pull_request.title, pull_request.html_url
                )),
                colour: Some(GREY.to_string()),
                ..Default::default()
            },
            PullRequestEvent::Opened {
                number,
                pull_request,
            } => SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] Pull Request Opened #{}: {}]({})\n{}",
                    event.repository.full_name,
                    number,
                    pull_request.title,
                    pull_request.html_url,
                    shorten_text(&pull_request.body.unwrap_or_default(), 450)
                )),
                colour: Some(ORANGE.to_string()),
                ..Default::default()
            },
            PullRequestEvent::Reopened {
                number,
                pull_request,
            } => SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!(
                    "#### [[{}] Pull Request Reopened #{}: {}]({})",
                    event.repository.full_name, number, pull_request.html_url, pull_request.title
                )),
                colour: Some(GREEN.to_string()),
                ..Default::default()
            },
            _ => return None,
        },
    };

    Some(sendable_embed)
}

#[cfg(test)]
mod tests {
    use super::{short_sha, shorten_single_line_text, shorten_text};

    #[test]
    fn shortening_respects_character_boundaries() {
        // non-ASCII used to panic on the byte slice
        assert_eq!(shorten_text("привет мир", 5), "пр...");
        assert_eq!(shorten_text("🎉🎉🎉🎉🎉", 4), "🎉...");
        // nothing to cut
        assert_eq!(shorten_text("привет", 10), "привет");
        assert_eq!(shorten_text("", 10), "");
        // used to underflow: first line shorter than the ellipsis
        assert_eq!(shorten_single_line_text("ab\ncd", 50), "ab...");
        assert_eq!(shorten_single_line_text("я\nещё", 50), "я...");
        assert_eq!(shorten_single_line_text("привет мир\nx", 5), "пр...");
        assert_eq!(shorten_single_line_text("привет", 10), "привет");
    }

    #[test]
    fn short_sha_survives_short_input() {
        assert_eq!(short_sha("0123456789abcdef"), "01234567");
        assert_eq!(short_sha("abc"), "abc");
        assert_eq!(short_sha(""), "");
    }
}
