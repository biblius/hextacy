pub mod router;

/// Utilities for working with http. The big boy of this module is the the [RestResponse][xhttp::response::RestResponse].
pub mod xhttp;

pub use cookie;
pub use http;
pub use mime;

/// A trait for hooking services up to application configurations. The usual application is simply
/// instantiating a service and calling a framework specific function to hook it up to a service.
pub trait Configure<T, C> {
    fn configure(state: &T, cfg: &mut C);
}
