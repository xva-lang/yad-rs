use crate::config::get_config;

pub(super) async fn create_db() -> Result<(), async_sqlite::Error> {
    let config = get_config();
    let conn = async_sqlite::rusqlite::Connection::open(config.database_path())?;

    create_if_not_exists(
        "pull_requests",
        r"
create table pull_requests
(
    id              integer       not null
        constraint pk_pull_requests
            primary key,
    number          int default 0 not null,
    repository      text          not null,
    status          integer       not null,
    merge_commit_id text,
    head_commit_id  text          not null,
    head_ref        text          not null,
    base_ref        text          not null,
    assignee        text,
    approved_by     text,
    priority        integer       not null,
    try_test        integer,
    rollup          integer       not null,
    squash          integer       not null,
    delegate        text,
    unique (id, repository)
);",
        &conn,
    )
    .await?;

    create_if_not_exists(
        "merges",
        r"
create table merges
(
    pull_request_id int
        constraint pk_tests
            primary key
        references pull_requests
            on delete cascade,
    status          int
);",
        &conn,
    )
    .await?;

    create_if_not_exists(
        "tests",
        r"
create table tests
(
    pull_request_id int
        constraint pk_tests
            primary key
        references pull_requests
            on delete cascade,
    status          int
);",
        &conn,
    )
    .await?;
    Ok(())
}

async fn create_if_not_exists(
    table_name: &str,
    ddl: &str,
    conn: &async_sqlite::rusqlite::Connection,
) -> Result<(), async_sqlite::Error> {
    if !table_exists(table_name, &conn).await? {
        match conn.execute(ddl, []) {
            Ok(_) => Ok(()),
            Err(e) => Err(async_sqlite::Error::Rusqlite(e)),
        }
    } else {
        Ok(())
    }
}

async fn table_exists(
    table_name: &str,
    conn: &async_sqlite::rusqlite::Connection,
) -> Result<bool, async_sqlite::Error> {
    let sql =
        format!("select * from sqlite_master where type = 'table' and name = '{table_name}';");
    match conn.query_row(&sql, [], |_| Ok(true)) {
        Ok(r) => Ok(r),
        Err(e) => match e {
            async_sqlite::rusqlite::Error::QueryReturnedNoRows => Ok(false),
            _ => Err(async_sqlite::Error::Rusqlite(e)),
        },
    }
}
