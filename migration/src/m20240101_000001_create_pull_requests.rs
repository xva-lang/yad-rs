use super::PullRequests;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create table main.pull_requests
        // (
        //     id              integer       not null
        //         constraint pk_pull_requests
        //             primary key,
        //     number          int default 0 not null,
        //     repository      text          not null,
        //     status          integer       not null,
        //     merge_commit_id text,
        //     head_commit_id  text          not null,
        //     head_ref        text          not null,
        //     base_ref        text          not null,
        //     assignee        text,
        //     approved_by     text,
        //     priority        integer       not null,
        //     try_test        integer,
        //     rollup          integer       not null,
        //     squash          integer       not null,
        //     delegate        text,
        //     unique (id, repository)
        // );
        manager
            .create_table(
                Table::create()
                    .table(PullRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PullRequests::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PullRequests::Number)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(PullRequests::Repository).text().not_null())
                    .col(ColumnDef::new(PullRequests::Status).integer().not_null())
                    .col(ColumnDef::new(PullRequests::MergeCommitId).text())
                    .col(ColumnDef::new(PullRequests::HeadCommitId).text().not_null())
                    .col(ColumnDef::new(PullRequests::HeadRef).text().not_null())
                    .col(ColumnDef::new(PullRequests::BaseRef).text().not_null())
                    .col(ColumnDef::new(PullRequests::Assignee).text())
                    .col(ColumnDef::new(PullRequests::ApprovedBy).text())
                    .col(ColumnDef::new(PullRequests::Priority).text().not_null())
                    .col(ColumnDef::new(PullRequests::TryTest).integer().not_null())
                    .col(ColumnDef::new(PullRequests::Rollup).text().not_null())
                    .col(ColumnDef::new(PullRequests::Squash).integer().not_null())
                    .col(ColumnDef::new(PullRequests::Delegate).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PullRequests::Table).to_owned())
            .await
    }
}
