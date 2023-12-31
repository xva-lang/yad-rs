/// Convenience macro for auto-`impl`ing the conversion trait for enum variants to and from SQL values
///
/// Derives [`Debug`] and generates `impl`s for [`ToSql`], [`FromSql`] and [`PartialEq`].
///
/// # Examples
/// ```
/// int_enum_sql! {
///     EnumName {
///         Variant1 => 1,
///         Variant2 => 2,
///         Variant3 => 3
///     }   
/// }
/// ```
macro_rules! int_enum_sql {

    ( $name:ident { $( $variant:tt => $value:tt ),+ } ) => {
        #[derive(Debug)]
        #[repr(i64)]
        pub(crate) enum $name {
            $(
                $variant
            ),+
        }

        // impl ToSql for $name {
        //     fn to_sql(&self) -> async_sqlite::rusqlite::Result<ToSqlOutput<'_>> {
        //         match self {
        //             $(
        //                 $name::$variant => Ok(ToSqlOutput::Owned(Value::Integer(
        //                     $value,
        //                 )))
        //             ),+
        //         }
        //     }
        // }

        // impl FromSql for $name {
        //     fn column_result(
        //         value: async_sqlite::rusqlite::types::ValueRef<'_>,
        //     ) -> async_sqlite::rusqlite::types::FromSqlResult<Self> {
        //         match value {
        //             async_sqlite::rusqlite::types::ValueRef::Integer(v) => match v {
        //                 $( $value => Ok($name::$variant), )+
        //                 _ => unreachable!(),
        //             },
        //             _ => unreachable!(),
        //         }
        //     }
        // }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                core::mem::discriminant(self) == core::mem::discriminant(other)
            }
        }
    };
}

const PULL_REQUEST_STATUS_PENDING: i64 = 0;
const PULL_REQUEST_STATUS_APPROVED: i64 = 1;
const PULL_REQUEST_STATUS_REJECTED: i64 = 2;
const PULL_REQUEST_STATUS_MERGED: i64 = 3;
const PULL_REQUEST_STATUS_CLOSED: i64 = 4;

const TEST_STATUS_WAITING: i64 = 0;
const TEST_STATUS_IN_PROGRESS: i64 = 1;
const TEST_STATUS_SUCCEEDED: i64 = 2;
const TEST_STATUS_FAILED: i64 = 3;

int_enum_sql! {
    TestStatus {
        Waiting => TEST_STATUS_WAITING,
        InProgress => TEST_STATUS_IN_PROGRESS,
        Succeeded => TEST_STATUS_SUCCEEDED,
        Failed => TEST_STATUS_FAILED
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct PullRequest {
    pub id: u64,
    pub number: i64,
    pub repository: String,
    pub status: entity::pull_requests::PullRequestStatus,
    merge_commit_id: Option<String>,
    pub head_commit_id: String,
    pub head_ref: String,
    base_ref: String,
    assignee: Option<String>,
    pub approved_by: Option<String>,
    priority: u64,
    try_test: bool,
    rollup: bool,
    squash: bool,
    delegate: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Test {
    pull_request_id: u64,
    status: TestStatus,
}

const MERGE_STATUS_WAITING: i64 = 0;
const MERGE_STATUS_STARTED: i64 = 1;
const MERGE_STATUS_FAILED: i64 = 2;

int_enum_sql! {
    MergeStatus {
        Waiting => MERGE_STATUS_WAITING,
        Started => MERGE_STATUS_STARTED,
        Failed => MERGE_STATUS_FAILED
    }
}
#[allow(dead_code)]
pub(crate) struct Merge {
    pull_request_id: u64,
    status: MergeStatus,
}

const CHECK_SUITE_STATUS_REQUESTED: i64 = 0;
const CHECK_SUITE_STATUS_PENDING: i64 = 1;
const CHECK_SUITE_STATUS_QUEUED: i64 = 2;
const CHECK_SUITE_STATUS_IN_PROGRESS: i64 = 3;
const CHECK_SUITE_STATUS_COMPLETED: i64 = 4;

int_enum_sql! {
    CheckSuiteStatus {
        Requested => CHECK_SUITE_STATUS_REQUESTED,
        Pending => CHECK_SUITE_STATUS_PENDING,
        Queued => CHECK_SUITE_STATUS_QUEUED,
        InProgress => CHECK_SUITE_STATUS_IN_PROGRESS,
        Completed => CHECK_SUITE_STATUS_COMPLETED
    }
}

pub(crate) struct CheckSuite {
    id: u64,
    pull_request_id: u64,
    status: CheckSuiteStatus,
}
