// @generated automatically by Diesel CLI.

diesel::table! {
    simple_models (id) {
        id -> Int4,
        some_param -> Varchar,
        other_param -> Int4,
    }
}

diesel::table! {
    test_users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(simple_models, test_users,);
