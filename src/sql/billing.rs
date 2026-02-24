#![allow(dead_code)]
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::billing)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Billing {
    pub id: i32,
    pub user_id: i32,
    pub amount_in_wallet: f64,
    pub amount_spent: f64,
    pub total_amount_spent: f64,
    pub average_hourly_consumption: f64,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::billing)]
pub struct NewBilling {
    pub user_id: i32,
    pub amount_in_wallet: f64,
    pub amount_spent: f64,
    pub total_amount_spent: f64,
    pub average_hourly_consumption: f64,
}
