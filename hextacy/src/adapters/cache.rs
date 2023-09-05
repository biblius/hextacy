#[cfg(any(feature = "cache-full", feature = "cache-redis"))]
pub mod redis;

#[cfg(any(feature = "cache-full", feature = "cache-inmem"))]
pub mod in_mem;

pub mod exports {
    #[cfg(any(feature = "cache-full", feature = "cache-redis",))]
    pub use deadpool_redis;
}
