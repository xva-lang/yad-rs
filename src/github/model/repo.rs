use super::User;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub(crate) struct Repository {
    pub id: u64,
    pub node_id: Option<String>,
    pub name: String,
    pub full_name: String,
    pub owner: Option<User>,
}
