use std::time::Duration;

use async_sqlite::{rusqlite::params, PoolBuilder};

use crate::{
    config::{get_config, Config},
    github::GithubClient,
    logging::error,
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
            continue;
        }

        let test_queue = pull_requests
            .into_iter()
            .filter(|x| x.status == PullRequestStatus::Approved)
            .collect::<Vec<_>>();

        if test_queue.len() == 0 {
            println!("No approved merges in queue");
            tokio::time::sleep(SLEEP_LENGTH).await;
            continue;
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

        tokio::time::sleep(SLEEP_LENGTH).await;
    }
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

pub(crate) async fn queue_merge(pull_request_id: u64) -> Result<(), async_sqlite::Error> {
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
