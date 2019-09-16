#[cfg(feature = "diesel_impl")]
use diesel_derives::Queryable;

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(Queryable))]
#[derive(Clone, PartialEq, Eq)]
pub struct Member {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub nickname: Option<String>,
}
