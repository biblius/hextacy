// @generated automatically by Diesel CLI.
#[rustfmt::skip]
diesel::table! {
    oauth (id) {
        id -> Varchar,
        user_id -> Varchar,
        access_token -> Varchar,
        refresh_token -> Nullable<Varchar>,
        provider -> Varchar,
        account_id -> Varchar,
        scope -> Varchar,
        revoked -> Bool,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sessions (id) {
        id -> Varchar,
        user_id -> Varchar,
        username -> Varchar,
        email -> Varchar,
        phone -> Nullable<Varchar>,
        role -> Varchar,
        csrf -> Varchar,
        oauth_token -> Nullable<Varchar>,
        auth_type -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Varchar,
        email -> Varchar,
        username -> Varchar,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        role -> Varchar,
        phone -> Nullable<Varchar>,
        password -> Nullable<Varchar>,
        otp_secret -> Nullable<Varchar>,
        frozen -> Bool,
        google_id -> Nullable<Varchar>,
        github_id -> Nullable<Varchar>,
        email_verified_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(oauth -> users (user_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(oauth, sessions, users,);
