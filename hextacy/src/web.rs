pub mod http;
pub mod middleware;
pub mod router;
pub mod ws;

use actix_web::web::ServiceConfig;

/// A trait for hooking services up to actix' [ServiceConfig]. The usual application is simply
/// instantiating a service, wrapping it in [Data][actix_web::web::Data] and calling `cfg.app_data()` with it.
pub trait Configure<T> {
    fn configure(state: &T, cfg: &mut ServiceConfig);
}
