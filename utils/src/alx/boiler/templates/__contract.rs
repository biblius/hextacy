use crate::error::Error;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(super) trait ServiceContract {}
