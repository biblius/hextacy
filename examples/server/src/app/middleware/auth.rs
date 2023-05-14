pub(crate) mod adapter;
pub(crate) mod interceptor;

use crate::{
    cache::contracts::AuthCacheAccess,
    db::{models::role::Role, repository::session::SessionRepository},
};
use adapter::*;
use hextacy::drivers::Connect;
use interceptor::*;
use std::{rc::Rc, sync::Arc};

impl<RepoDriver, CacheDriver, CacheConn, Cache, RepoConn, Session>
    AuthenticationGuardInner<
        AuthMwRepo<RepoDriver, RepoConn, Session>,
        AuthMwCache<CacheDriver, CacheConn, Cache>,
    >
where
    CacheDriver: Connect<Connection = CacheConn> + Send + Sync,
    RepoDriver: Connect<Connection = RepoConn> + Send + Sync,
    Cache: AuthCacheAccess<CacheConn> + Send + Sync,
    Session: SessionRepository<RepoConn> + Send + Sync,
{
    pub fn new(repository: Arc<RepoDriver>, cache: Arc<CacheDriver>, role: Role) -> Self {
        Self {
            cache: AuthMwCache::new(cache),
            repository: AuthMwRepo::new(repository),
            auth_level: role,
        }
    }
}

impl<RepoDriver, CacheDriver, CacheConn, Cache, RepoConn, Session>
    AuthenticationGuard<
        AuthMwRepo<RepoDriver, RepoConn, Session>,
        AuthMwCache<CacheDriver, CacheConn, Cache>,
    >
where
    CacheDriver: Connect<Connection = CacheConn> + Send + Sync,
    RepoDriver: Connect<Connection = RepoConn> + Send + Sync,
    Cache: AuthCacheAccess<CacheConn> + Send + Sync,
    Session: SessionRepository<RepoConn> + Send + Sync,
{
    pub fn new(repository: Arc<RepoDriver>, cache: Arc<CacheDriver>, role: Role) -> Self {
        Self {
            inner: Rc::new(AuthenticationGuardInner::new(repository, cache, role)),
        }
    }
}
