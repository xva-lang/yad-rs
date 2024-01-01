use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, Set};

use crate::{
    config::get_config,
    db::get_db,
    github::{
        create_issue_comment,
        model::{pulls::PullRequest, repo::Repository},
        GithubClient,
    },
    logging::{error, info},
    queue::enqueue_merge,
    routes::IssueCommentPayload,
};

use entity::pull_requests::PullRequestStatus;

const DEFAULT_PING_MESSAGE: &str = "Hi @{{COMMENTER}}! Yes, I'm still alive!";
const PING_MESSAGE_COMMENTER_PATTERN: &str = "{{COMMENTER}}";
pub(crate) async fn ping(ic: &IssueCommentPayload) {
    let owner = &ic.repository.owner.as_ref().unwrap().login;
    let repo = &ic.repository.name;
    let commenter = &ic.issue.user.login;
    let config = get_config();
    let ping_message = match config.actions {
        Some(ref a) => match a.ping {
            Some(ref p) => match p.message {
                Some(ref msg) => msg,
                None => DEFAULT_PING_MESSAGE,
            },
            None => DEFAULT_PING_MESSAGE,
        },
        None => DEFAULT_PING_MESSAGE,
    };

    info(
        format!("@{commenter} has tried to check whether service is still alive"),
        Some(&config),
    );

    match create_issue_comment(
        owner,
        repo,
        ic.issue.number,
        &ping_message.replace(PING_MESSAGE_COMMENTER_PATTERN, &commenter),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => panic!("{e}"),
    }
}

pub(crate) async fn set_assignee(ic: &IssueCommentPayload, assignee: Option<String>) {
    let owner = &ic.repository.owner.as_ref().unwrap().login;
    let repo = &ic.repository.name;
    let commenter = &ic.issue.user.login;
    let issue_number = ic.issue.number;
    let config = get_config();

    // If the value is none, the commenter has issued the "claim" command - i.e. they are assigning themselves.
    let assignee = match &assignee {
        Some(v) => v.as_str(),
        None => commenter.as_str(),
    };

    // Assign the specified user and report the action as an issue comment
    let client = GithubClient::new(config.access_token());
    if let Err(e) = client
        .add_assignee_to_issue(owner, repo, issue_number, assignee)
        .await
    {
        error(
            format!("Failed to add assignee to issue #{issue_number}. Extended error: {e}"),
            Some(&config),
        )
    }
}

pub(crate) async fn remove_assignee(ic: &IssueCommentPayload) {
    let owner = &ic.repository.owner.as_ref().unwrap().login;
    let repo = &ic.repository.name;
    let commenter = &ic.issue.user.login;
    let issue_number = ic.issue.number;
    let config = get_config();

    // Check that the user that issued the command is one of the assignees already
    // If a user that issued this command is not already an assignee then no-op
    let client = GithubClient::new(config.access_token());

    let mut should_delete_assignee = false;
    match client.list_issue_assignees(owner, repo, issue_number).await {
        Ok(assignees) => {
            if assignees.iter().map(|x| &x.login).any(|x| x == commenter) {
                should_delete_assignee = true;
            }
        }
        Err(e) => error(
            format!("Failed to retrieve issue. Extended error: {e}"),
            Some(&config),
        ),
    }

    if should_delete_assignee {
        if let Err(e) = client
            .delete_assignee(owner, repo, issue_number, &commenter)
            .await
        {
            error(
                format!("Failed to delete assignee on issue #{issue_number}. Extended error: {e}"),
                Some(&config),
            );
        }
    }
}

pub(crate) async fn save_pull_to_db(pr: PullRequest, repo: Repository) -> Result<(), DbErr> {
    let row = entity::pull_requests::ActiveModel {
        id: Set(pr.id),
        number: Set(pr.number),
        repository: Set(repo.full_name),
        status: Set(PullRequestStatus::Pending),
        merge_commit_id: Set(pr.merge_commit_sha),
        head_commit_id: Set(pr.head.sha),
        head_ref: Set(pr.head.label.unwrap()),
        base_ref: Set(pr.base.label.unwrap()),
        assignee: Set(pr.assignee.map_or(None, |x| Some(x.login))),
        approved_by: Set(None),
        priority: Set(0),
        try_test: Set(false),
        rollup: Set(0),
        squash: Set(false),
        delegate: Set(None),
    };

    row.insert(&get_db().await?).await?;
    Ok(())
}

pub(crate) async fn set_pull_request_status(
    pr_id: u64,
    status: PullRequestStatus,
) -> Result<(), DbErr> {
    use entity::pull_requests::Entity as PullRequests;
    let db = get_db().await?;
    //     let pear: Option<fruit::Model> = Fruit::find_by_id(28).one(db).await?;

    // // Into ActiveModel
    // let mut pear: fruit::ActiveModel = pear.unwrap().into();

    // // Update name attribute
    // pear.name = Set("Sweet pear".to_owned());

    // // SQL: `UPDATE "fruit" SET "name" = 'Sweet pear' WHERE "id" = 28`
    // let pear: fruit::Model = pear.update(db).await?;

    let pr: Option<entity::pull_requests::Model> = PullRequests::find_by_id(pr_id).one(&db).await?;
    let mut pr: entity::pull_requests::ActiveModel = pr.unwrap().into();

    pr.status = Set(status);

    match pr.update(&db).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
    //     let config = get_config();
    //     let client = async_sqlite::ClientBuilder::new()
    //         .path(config.database_path())
    //         .open()
    //         .await
    //         .unwrap();

    //     const STATEMENT: &str = r#"
    // update pull_requests
    // set status = ?1
    // where id = ?2"#;

    //     client
    //         .conn(
    //             move |conn| match conn.execute(STATEMENT, params![status, pr_id]) {
    //                 Ok(_) => Ok(()),
    //                 Err(e) => Err(e),
    //             },
    //         )
    //         .await
}

pub(crate) async fn set_pull_request_approved(
    pr_id: u64,
    approved_by: String,
) -> Result<(), DbErr> {
    use entity::pull_requests::Entity as PullRequests;
    let db = get_db().await?;

    let pr: Option<entity::pull_requests::Model> = PullRequests::find_by_id(pr_id).one(&db).await?;
    let mut pr: entity::pull_requests::ActiveModel = pr.unwrap().into();

    pr.approved_by = Set(Some(approved_by));

    match pr.update(&db).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub(crate) async fn approve_pull(ic: &IssueCommentPayload) {
    let owner = &ic.repository.owner.as_ref().unwrap().login;
    let repo = &ic.repository.name;
    let commenter = &ic.issue.user.login;
    let issue_number = ic.issue.number;

    let config = get_config();

    let client = GithubClient::new(config.access_token());
    let (pull_number, commit_id, pull_id) = match client
        .get_pull_request_from_issue_number(owner, repo, issue_number)
        .await
    {
        Ok(inner) => match inner {
            Some(pr) => (pr.number, pr.head.sha, pr.id),
            None => {
                error(
                    format!("No pull request for issue #{issue_number}"),
                    Some(&config),
                );
                return;
            }
        },
        Err(e) => {
            error(
                format!("Failed to retrieve pull request for issue #{issue_number}. {e}"),
                Some(&config),
            );
            return;
        }
    };

    if let Err(e) = set_pull_request_approved(pull_id, commenter.clone()).await {
        error(
            format!("Failed to save approved status for pull request #{pull_number}. {e}"),
            Some(&config),
        );
        return;
    }

    match enqueue_merge(pull_id).await {
        Ok(_) => {}
        Err(e) => {
            error(
                format!("Failed to add approved review to pull request #{pull_number}. {e}"),
                Some(&config),
            );
        }
    }

    let body = format!(
        r"
:pushpin: Commit {commit_id} has been approved by `{commenter}`

It is now in the queue for this repository."
    );
    match client
        .create_issue_comment(owner, repo, issue_number, &body)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            error(
                format!("Failed to create issue comment for approved pull. {e}"),
                Some(&config),
            );
        }
    }
}
