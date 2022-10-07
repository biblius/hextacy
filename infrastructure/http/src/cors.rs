use actix_cors::Cors;
use actix_web::http::header::*;

pub fn setup_cors(allowed_origins: Vec<&str>, expose_headers: Vec<&str>) -> Cors {
    let mut cors = Cors::default()
        .supports_credentials()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![
            AUTHORIZATION,
            ACCEPT,
            CONTENT_TYPE,
            ORIGIN,
            ACCESS_CONTROL_REQUEST_METHOD,
            HeaderName::from_static("x-csrf-token"),
        ])
        .expose_headers(expose_headers);
    for origin in allowed_origins {
        cors = cors.allowed_origin(origin);
    }
    cors.max_age(3600)
}
