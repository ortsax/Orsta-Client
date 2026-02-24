// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        passkey -> Nullable<Text>,
        eakey -> Text,
    }
}

diesel::table! {
    user_property (id) {
        id -> Integer,
        user_id -> Integer,
        instance_status -> Text,
        instance_usage -> Double,
        api_key_active -> Bool,
    }
}

diesel::table! {
    instances (id) {
        id -> Integer,
        user_id -> Integer,
        instances_count -> Integer,
        expected_consumption -> Double,
        instances_overall_consumption -> Double,
    }
}

diesel::table! {
    billing (id) {
        id -> Integer,
        user_id -> Integer,
        amount_in_wallet -> Double,
        amount_spent -> Double,
        total_amount_spent -> Double,
        average_hourly_consumption -> Double,
    }
}

diesel::joinable!(user_property -> users (user_id));
diesel::joinable!(instances -> users (user_id));
diesel::joinable!(billing -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(users, user_property, instances, billing,);
