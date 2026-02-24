use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A WhatsApp bot instance managed by Orsta.
///
/// Availability is restricted to the 180 countries supported by WhatsApp.
/// `country_code` should be a valid ISO 3166-1 alpha-2 code for one of those
/// countries. `active` is stored as `0` (inactive) or `1` (active); billing
/// accrues only while `active = 1`.
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::instances)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Instance {
    pub id: i32,
    pub user_id: i32,
    pub country_code: String,
    pub phone_number: String,
    /// 0 = inactive, 1 = active. Billing only runs while this is 1.
    pub active: i32,
    /// Unix timestamp (seconds) when the instance was created.
    pub created_at: i64,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::instances)]
pub struct NewInstance {
    pub user_id: i32,
    pub country_code: String,
    pub phone_number: String,
}
