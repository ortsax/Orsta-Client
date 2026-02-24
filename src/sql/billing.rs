use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Pricing constants
// ---------------------------------------------------------------------------

/// Base rate: $0.48 per hour expressed in cents.
pub const HOURLY_RATE_CENTS: i64 = 48;

/// 30% discount applied to new users for their first two months.
pub const PROMOTION_DISCOUNT_PERCENT: i64 = 30;

/// Duration of the new-user promotion in seconds (~2 calendar months).
pub const PROMOTION_DURATION_SECS: i64 = 2 * 30 * 24 * 3_600;

// ---------------------------------------------------------------------------
// Database model
// ---------------------------------------------------------------------------

/// A single continuous active window for a WhatsApp bot instance.
///
/// `ended_at` is `NULL` while the instance is still running. When the
/// instance is deactivated the field is set and `amount_cents` is computed
/// via [`calculate_charge_cents`].
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::billing_records)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BillingRecord {
    pub id: i32,
    pub instance_id: i32,
    pub user_id: i32,
    /// Unix timestamp when this billing window opened.
    pub started_at: i64,
    /// Unix timestamp when this billing window closed, or `None` if still open.
    pub ended_at: Option<i64>,
    /// Total charge for this window in cents. 0 while the window is open.
    pub amount_cents: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::billing_records)]
pub struct NewBillingRecord {
    pub instance_id: i32,
    pub user_id: i32,
    pub started_at: i64,
}

// ---------------------------------------------------------------------------
// Billing logic
// ---------------------------------------------------------------------------

/// Calculates the charge in cents for one active billing window.
///
/// ## Rules
/// - Rate: **$0.48 / hour** (48 cents / hour), charged pro-rata per second.
/// - New users receive a **30 % discount** for their first 2 months.
///
/// ## Arguments
/// - `duration_secs` – How long the instance was active (seconds).
/// - `user_created_at` – Unix timestamp when the user registered.
/// - `billing_period_start` – Unix timestamp for the start of this window.
///   Used to determine whether the user is still within the promotional period.
///
/// ## Returns
/// Charge in integer cents (rounded to the nearest cent).
pub fn calculate_charge_cents(
    duration_secs: i64,
    user_created_at: i64,
    billing_period_start: i64,
) -> i32 {
    let hours = duration_secs as f64 / 3_600.0;
    let base_cents = (hours * HOURLY_RATE_CENTS as f64).round() as i64;

    let within_promotion =
        billing_period_start - user_created_at <= PROMOTION_DURATION_SECS;

    let final_cents = if within_promotion {
        (base_cents as f64 * (100 - PROMOTION_DISCOUNT_PERCENT) as f64 / 100.0).round() as i64
    } else {
        base_cents
    };

    final_cents as i32
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a base timestamp representing "now" (arbitrary fixed value).
    const BASE: i64 = 1_700_000_000;

    #[test]
    fn full_hour_no_discount() {
        // User registered more than 2 months before billing start → no discount.
        let user_created_at = BASE - PROMOTION_DURATION_SECS - 1;
        let charge = calculate_charge_cents(3_600, user_created_at, BASE);
        assert_eq!(charge, 48);
    }

    #[test]
    fn full_hour_with_promotion() {
        // User registered within the 2-month promotion window.
        let user_created_at = BASE - 1;
        let charge = calculate_charge_cents(3_600, user_created_at, BASE);
        // 48 cents * 70 % = 33.6 → rounds to 34
        assert_eq!(charge, 34);
    }

    #[test]
    fn zero_duration_no_charge() {
        let user_created_at = BASE - 1;
        let charge = calculate_charge_cents(0, user_created_at, BASE);
        assert_eq!(charge, 0);
    }

    #[test]
    fn half_hour_no_discount() {
        let user_created_at = BASE - PROMOTION_DURATION_SECS - 1;
        let charge = calculate_charge_cents(1_800, user_created_at, BASE);
        // 0.5 h * 48 = 24 cents
        assert_eq!(charge, 24);
    }

    #[test]
    fn exactly_at_promotion_boundary_gets_discount() {
        // Registered exactly at the start of the 2-month window.
        let user_created_at = BASE - PROMOTION_DURATION_SECS;
        let charge = calculate_charge_cents(3_600, user_created_at, BASE);
        assert_eq!(charge, 34);
    }

    #[test]
    fn one_second_past_promotion_no_discount() {
        let user_created_at = BASE - PROMOTION_DURATION_SECS - 1;
        let charge = calculate_charge_cents(3_600, user_created_at, BASE);
        assert_eq!(charge, 48);
    }
}
