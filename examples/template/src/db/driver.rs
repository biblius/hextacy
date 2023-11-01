use async_trait::async_trait;
use hextacy::{Driver, DriverError};
use sea_orm::DatabaseConnection;
use sea_orm::{ConnectOptions, Database};

#[cfg(test)]
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, IntoActiveModel,
    ModelTrait, PrimaryKeyTrait,
};

/// Contains a connection pool for postgres with sea-orm. An instance of this
/// should be shared through the app with Arcs
#[derive(Debug, Clone)]
pub struct SeaormDriver {
    pool: DatabaseConnection,
}

impl SeaormDriver {
    pub async fn new(url: &str) -> Self {
        let pool = Database::connect(ConnectOptions::new(url))
            .await
            .expect("Could not establish database connection");
        Self { pool }
    }
}

#[async_trait]
impl Driver for SeaormDriver {
    type Connection = DatabaseConnection;
    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.pool.connect().await
    }
}

// Helper implementations
#[cfg(test)]
impl SeaormDriver {
    pub async fn insert<R, M, A, C, E>(&self, conn: &C, model: A) -> Result<R, DbErr>
    where
        C: ConnectionTrait,
        E: EntityTrait<Model = M>,
        M: ModelTrait<Entity = E> + IntoActiveModel<A> + FromQueryResult,
        A: ActiveModelTrait<Entity = E>,
        R: From<M>,
    {
        E::insert(model)
            .exec_with_returning(conn)
            .await
            .map(R::from)
    }

    pub async fn get_by_id<R, M, Id, E, C>(&self, conn: &C, id: Id) -> Result<Option<R>, DbErr>
    where
        C: ConnectionTrait + Send,
        E: EntityTrait<Model = M>,
        M: ModelTrait<Entity = E> + Send + FromQueryResult + 'static,
        <E::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Id>,
        R: From<M>,
    {
        E::find_by_id(id).one(conn).await.map(|o| o.map(R::from))
    }

    pub async fn delete<M, Id, E, C>(&self, conn: &C, id: Id) -> Result<u64, DbErr>
    where
        C: ConnectionTrait + Send,
        E: EntityTrait<Model = M>,
        M: ModelTrait<Entity = E> + Send + FromQueryResult + 'static,
        <E::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Id>,
    {
        E::delete_by_id(id)
            .exec(conn)
            .await
            .map(|res| res.rows_affected)
    }
}
