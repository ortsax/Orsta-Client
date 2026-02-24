#![allow(dead_code)]
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::user_property)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserProperty {
    pub id: i32,
    pub user_id: i32,
    pub instance_status: String,
    pub instance_usage: f64,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::user_property)]
pub struct NewUserProperty {
    pub user_id: i32,
    pub instance_status: String,
    pub instance_usage: f64,
}
