// @generated automatically by Diesel CLI.

diesel::table! {
    sessions (id) {
        id -> Varchar,
        user_id -> Varchar,
        username -> Varchar,
        user_role -> Varchar,
        frozen -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        soft_expires_at -> Timestamptz,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Varchar,
        email -> Varchar,
        username -> Varchar,
        role -> Varchar,
        password -> Nullable<Varchar>,
        otp_secret -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        google_id -> Nullable<Varchar>,
        github_id -> Nullable<Varchar>,
        frozen -> Bool,
        email_verified_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    sessions,
    users,
);
