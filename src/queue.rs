use entity::{
    merges::{Entity as MergesEntity, Model as MergesModel},
    pull_requests::{Entity as PullRequestsEntity, Model as PullRequestsModel},
};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, Set};
use std::time::Duration;

use crate::{
    config::{get_config, Config},
    db::get_db,
    github::GithubClient,
    logging::{error, info},
};

pub(crate) async fn queue_server() {
    let config = get_config();
    const SLEEP_LENGTH: Duration = Duration::from_millis(5000);

    let gh_client = GithubClient::new(&config.access_token());
    loop {
        handle_merge_queue(&gh_client, &config).await.unwrap();
        tokio::time::sleep(SLEEP_LENGTH).await;
    }
}

async fn handle_merge_queue(client: &GithubClient<'_>, config: &Config) -> Result<(), DbErr> {
    let db = get_db().await?;

    let pulls_with_merges: Vec<(PullRequestsModel, Vec<MergesModel>)> = PullRequestsEntity::find()
        .find_with_related(MergesEntity)
        .all(&get_db().await?)
        .await?;

    for (pr, merges) in pulls_with_merges {
        if merges.len() == 0 || merges.len() > 1 {
            continue;
        }

        let parts = pr.repository.split("/").collect::<Vec<_>>();
        let (owner, repo) = (parts.get(0).unwrap(), parts.get(1).unwrap());
        let pull_number = pr.number;
        let head_ref = pr.head_ref;
        let approver = pr.approved_by.unwrap();

        info(
            format!("Starting merge for pull request #{pull_number}"),
            Some(config),
        );
        //             "update merges set status = ?1 where pull_request_id = ?2",

        for merge in merges {
            let mut update_merge: entity::merges::ActiveModel = merge.clone().into();
            update_merge.status = Set(entity::merges::MergeStatus::Started);
            update_merge = match update_merge.update(&db).await {
                Ok(r) => r.into(),
                Err(e) => return Err(e),
            };

            if let Err(e) = client
                .merge_pull(owner, repo, pr.number as u64, &head_ref, &approver)
                .await
            {
                error(format!("Failed to merge pull request. {e}"), Some(&config));

                update_merge.status = Set(entity::merges::MergeStatus::Failed);
                update_merge.update(&db).await?;
            } else {
                update_merge.delete(&db).await?;
            }
        }
    }

    Ok(())
}

// async fn get_approved_pull_requests(
//     pool: &async_sqlite::Pool,
// ) -> Result<Vec<PullRequest>, async_sqlite::Error> {
//     let sql = r"
// select pr.*
// from pull_requests pr
// left join tests t on pr.id = t.pull_request_id
// where
//     t.pull_request_id is null and
//     pr.status = ?1";

//     pool.conn(|conn| {
//         let mut stmt = conn.prepare(sql).unwrap();
//         Ok(stmt
//             .query_map([1], |r| Ok(PullRequest::from(r)))
//             .unwrap()
//             .into_iter()
//             .map(|x| x.unwrap())
//             .collect::<Vec<PullRequest>>())
//     })
//     .await
// }

pub(crate) async fn enqueue_merge(pull_request_id: u64) -> Result<(), DbErr> {
    let row = entity::merges::ActiveModel {
        pull_request_id: Set(pull_request_id),
        status: Set(entity::merges::MergeStatus::Waiting),
    };

    row.insert(&get_db().await?).await?;
    Ok(())
}
