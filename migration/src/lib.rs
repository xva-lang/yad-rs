pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_pull_requests;
mod m20240101_101620_create_merges;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_pull_requests::Migration),
            Box::new(m20240101_101620_create_merges::Migration),
        ]
    }
}

#[derive(DeriveIden)]
pub(crate) enum Merges {
    Table,
    PullRequestId,
    Status,
}

#[derive(DeriveIden)]
pub(crate) enum PullRequests {
    Table,
    Id,
    Number,
    Repository,
    Status,
    #[sea_orm(iden = "merge_commit_id")]
    MergeCommitId,
    #[sea_orm(iden = "head_commit_id")]
    HeadCommitId,
    #[sea_orm(iden = "head_ref")]
    HeadRef,
    #[sea_orm(iden = "base_ref")]
    BaseRef,
    Assignee,
    #[sea_orm(iden = "approved_by")]
    ApprovedBy,
    Priority,
    #[sea_orm(iden = "try_test")]
    TryTest,
    Rollup,
    Squash,
    Delegate,
}
