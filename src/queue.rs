use std::time::Duration;

use async_sqlite::{rusqlite::params, PoolBuilder};

use crate::{
    config::{get_config, Config},
    github::GithubClient,
    logging::{error, info},
    model::{MergeStatus, PullRequest, PullRequestStatus},
};

pub(crate) async fn queue_server() {
    let config = get_config();
    const SLEEP_LENGTH: Duration = Duration::from_millis(5000);

    let mut gh_client = GithubClient::new(&config.access_token());
    let pool = PoolBuilder::new()
        .path(config.database_path())
        .open()
        .await
        .unwrap();

    loop {
        let pull_requests = match get_approved_pull_requests(&pool).await {
            Ok(v) => v,
            Err(e) => {
                error(
                    format!("Failed to retrieve merge queue. Extended error: {e}"),
                    Some(&config),
                );

                tokio::time::sleep(SLEEP_LENGTH).await;
                continue;
            }
        };

        if pull_requests.len() == 0 {
            // info(), config)
            println!("Nothing to do");
            tokio::time::sleep(SLEEP_LENGTH).await;
            //    continue;
        }

        let test_queue = pull_requests
            .into_iter()
            .filter(|x| x.status == PullRequestStatus::Approved)
            .collect::<Vec<_>>();

        if test_queue.len() == 0 {
            println!("No approved merges in queue");
            tokio::time::sleep(SLEEP_LENGTH).await;
            //  continue;
        }

        for test in test_queue {
            let parts = test.repository.split("/").collect::<Vec<_>>();
            let (owner, repo) = (parts.get(0).unwrap(), parts.get(1).unwrap());
            let head_sha = &test.head_commit_id;
            match pool
                .conn(move |conn| {
                    conn.execute(
                        "insert into tests (pull_request_id, status) values (?1, ?2)",
                        params![test.id, 0],
                    )
                })
                .await
            {
                Ok(r) => {
                    // if let Err(e) = gh_client.create_check(owner, repo, head_sha).await {
                    //     error(
                    //         format!("Failed to create CI check. Extended error: {e}"),
                    //         Some(&config),
                    //     );
                    // };
                }
                Err(e) => {
                    error(
                        format!("Failed to insert into test table. Extended error: {e}"),
                        Some(&config),
                    );
                    continue;
                }
            }
        }
        handle_merge_queue(&pool, &gh_client, &config)
            .await
            .unwrap();
        tokio::time::sleep(SLEEP_LENGTH).await;
    }
}

async fn handle_merge_queue(
    pool: &async_sqlite::Pool,
    client: &GithubClient<'_>,
    config: &Config,
) -> Result<(), async_sqlite::Error> {
    let sql = r"
select pr.* 
from pull_requests pr 
left join merges m on pr.id = m.pull_request_id
where
    m.pull_request_id is not null and 
    m.status = ?1";
    let merge_prs = pool
        .conn(|conn| {
            let mut stmt = conn.prepare(sql).unwrap();
            Ok(stmt
                .query_map([MergeStatus::Waiting], |r| Ok(PullRequest::from(r)))
                .unwrap()
                .into_iter()
                .map(|x| x.unwrap())
                .collect::<Vec<PullRequest>>())
        })
        .await?;

    for pr in merge_prs {
        let parts = pr.repository.split("/").collect::<Vec<_>>();
        let (owner, repo) = (parts.get(0).unwrap(), parts.get(1).unwrap());
        let pr_id = pr.id;
        let pull_number = pr.number;
        let head_ref = pr.head_ref;
        let approver = pr.approved_by.unwrap();

        info(
            format!("Starting merge for pull request #{pull_number}"),
            Some(&config),
        );

        pool.conn(move |conn| {
            conn.execute(
                "update merges set status = ?1 where pull_request_id = ?2",
                params![MergeStatus::Started, pr_id],
            )
        })
        .await
        .unwrap();

        if let Err(e) = client
            .merge_pull(owner, repo, pr.number as u64, &head_ref, &approver)
            .await
        {
            error(format!("Failed to merge pull request. {e}"), Some(&config));
            pool.conn(move |conn| {
                conn.execute(
                    "update merges set status = ?1 where pull_request_id = ?2",
                    params![MergeStatus::Failed, pr_id],
                )
            })
            .await
            .unwrap();
        } else {
            pool.conn(move |conn| {
                conn.execute(
                    "delete from merges where pull_request_id = ?1",
                    params![pr_id],
                )
            })
            .await
            .unwrap();
        }
    }

    Ok(())
}

async fn get_approved_pull_requests(
    pool: &async_sqlite::Pool,
) -> Result<Vec<PullRequest>, async_sqlite::Error> {
    let sql = r"
select pr.* 
from pull_requests pr 
left join tests t on pr.id = t.pull_request_id 
where 
    t.pull_request_id is null and 
    pr.status = ?1";

    pool.conn(|conn| {
        let mut stmt = conn.prepare(sql).unwrap();
        Ok(stmt
            .query_map([1], |r| Ok(PullRequest::from(r)))
            .unwrap()
            .into_iter()
            .map(|x| x.unwrap())
            .collect::<Vec<PullRequest>>())
    })
    .await
}

pub(crate) async fn enqueue_merge(pull_request_id: u64) -> Result<(), async_sqlite::Error> {
    let sql = "insert into merges (pull_request_id, status) values (?1, ?2)";
    let config = get_config();
    let pool = PoolBuilder::new()
        .path(config.database_path())
        .open()
        .await?;

    match pool
        .conn(move |conn| conn.execute(sql, params![pull_request_id, MergeStatus::Waiting]))
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
