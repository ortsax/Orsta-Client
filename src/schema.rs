// @generated automatically by Diesel CLI.

diesel::table! {
    billing_records (id) {
        id -> Integer,
        instance_id -> Integer,
        user_id -> Integer,
        started_at -> BigInt,
        ended_at -> Nullable<BigInt>,
        amount_cents -> Integer,
    }
}

diesel::table! {
    instances (id) {
        id -> Integer,
        user_id -> Integer,
        country_code -> Text,
        phone_number -> Text,
        active -> Integer,
        created_at -> BigInt,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        created_at -> BigInt,
    }
}

diesel::joinable!(billing_records -> instances (instance_id));
diesel::joinable!(billing_records -> users (user_id));
diesel::joinable!(instances -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(billing_records, instances, users,);
