#![allow(dead_code)]
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::instances)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Instance {
    pub id: i32,
    pub user_id: i32,
    pub instances_count: i32,
    pub expected_consumption: f64,
    pub instances_overall_consumption: f64,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::instances)]
pub struct NewInstance {
    pub user_id: i32,
    pub instances_count: i32,
    pub expected_consumption: f64,
    pub instances_overall_consumption: f64,
}
