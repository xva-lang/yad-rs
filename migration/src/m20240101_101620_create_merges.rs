use super::{Merges, PullRequests};

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        //         create table main.merges
        // (
        //     pull_request_id int
        //         constraint pk_tests
        //             primary key
        //         references main.pull_requests
        //             on delete cascade,
        //     status          int
        // );

        manager
            .create_table(
                Table::create()
                    .table(Merges::Table)
                    .col(ColumnDef::new(Merges::PullRequestId).integer().not_null())
                    .col(ColumnDef::new(Merges::Status).integer().not_null())
                    .primary_key(Index::create().name("pk_merges").col(Merges::PullRequestId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_merges_pull_requests")
                            .from(Merges::Table, Merges::PullRequestId)
                            .to(PullRequests::Table, PullRequests::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Merges::Table).to_owned())
            .await
    }
}
