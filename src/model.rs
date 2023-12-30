use async_sqlite::rusqlite::{
    types::{FromSql, ToSqlOutput, Value},
    Row, ToSql,
};
use serde::Deserialize;

#[derive(Debug)]
pub(crate) enum PullRequestStatus {
    Pending,
    Approved,
    Rejected,
}

const PULL_REQUEST_STATUS_PENDING: i64 = 0;
const PULL_REQUEST_STATUS_APPROVED: i64 = 1;
const PULL_REQUEST_STATUS_REJECTED: i64 = 2;

impl FromSql for PullRequestStatus {
    fn column_result(
        value: async_sqlite::rusqlite::types::ValueRef<'_>,
    ) -> async_sqlite::rusqlite::types::FromSqlResult<Self> {
        match value {
            async_sqlite::rusqlite::types::ValueRef::Integer(v) => match v {
                PULL_REQUEST_STATUS_PENDING => Ok(Self::Pending),
                PULL_REQUEST_STATUS_APPROVED => Ok(Self::Approved),
                PULL_REQUEST_STATUS_REJECTED => Ok(Self::Rejected),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

impl ToSql for PullRequestStatus {
    fn to_sql(&self) -> async_sqlite::rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            PullRequestStatus::Pending => Ok(ToSqlOutput::Owned(Value::Integer(
                PULL_REQUEST_STATUS_PENDING,
            ))),
            PullRequestStatus::Approved => Ok(ToSqlOutput::Owned(Value::Integer(
                PULL_REQUEST_STATUS_APPROVED,
            ))),
            PullRequestStatus::Rejected => Ok(ToSqlOutput::Owned(Value::Integer(
                PULL_REQUEST_STATUS_REJECTED,
            ))),
        }
    }
}
#[derive(Debug)]
pub(crate) struct PullRequest {
    id: u64,
    repository: String,
    status: PullRequestStatus,
    merge_commit_id: String,
    head_commit_id: String,
    head_ref: String,
    base_ref: String,
    assignee: Option<String>,
    approved_by: Option<String>,
    priority: u64,
    try_test: bool,
    rollup: bool,
    squash: bool,
    delegate: Option<String>,
}

impl From<&Row<'_>> for PullRequest {
    fn from(value: &Row<'_>) -> Self {
        Self {
            id: value.get(0).unwrap(),
            repository: value.get(1).unwrap(),
            status: value.get(2).unwrap(),
            merge_commit_id: value.get(3).unwrap(),
            head_commit_id: value.get(4).unwrap(),
            head_ref: value.get(5).unwrap(),
            base_ref: value.get(6).unwrap(),
            assignee: value.get(7).unwrap(),
            approved_by: value.get(8).unwrap(),
            priority: value.get(9).unwrap(),
            try_test: value.get(10).unwrap(),
            rollup: value.get(11).unwrap(),
            squash: value.get(12).unwrap(),
            delegate: value.get(13).unwrap(),
        }
    }
}
