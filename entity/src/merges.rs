//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum MergeStatus {
    #[sea_orm(num_value = 0)]
    Waiting,
    #[sea_orm(num_value = 1)]
    Started,
    #[sea_orm(num_value = 2)]
    Failed,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "merges")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub pull_request_id: u64,
    pub status: MergeStatus,
}

impl ActiveModelBehavior for ActiveModel {}

// Many to one relationship
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    PullRequest,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::PullRequest => Entity::belongs_to(super::pull_requests::Entity)
                .from(Column::PullRequestId)
                .to(super::pull_requests::Column::Id)
                .into(),
        }
    }
}

use super::pull_requests::Entity as PullRequestsEntity;
impl Related<PullRequestsEntity> for Entity {
    fn to() -> RelationDef {
        Relation::PullRequest.def()
    }
}
