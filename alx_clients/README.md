# ALX Clients

Contains clients for communicating with database servers and more. DB clients are intended to be used with the ALX storage module.
Provides basic functionality for building connection pools and establishing direct connections.

## Feature flags

- postgres
- redis
- mongo
- oauth
- email

## Setup

To specify the connection parameters a `.env` file in the project root directory is required with the following configuration:

```bash
# Postgres

PG_USER =
PG_PASSWORD =
PG_HOST =
PG_PORT =
PG_DATABASE =
POSTGRES_URL = "postgresql://${PG_USER}:${PG_PASSWORD}@${PG_HOST}:${PG_PORT}/${PG_DATABASE}"
# Optional, defaults to 8
PG_POOL_SIZE =

# Redis

RD_USER =
RD_PASSWORD =
RD_HOST =
RD_PORT =
RD_DATABASE =
REDIS_URL = "redis://${RD_HOST}"
# Optional, defaults to 8
RD_POOL_SIZE = 

# Mongo

MONGO_USER =
MONGO_PASSWORD =
MONGO_HOST =
MONGO_PORT =
MONGO_DATABASE =
MONGO_APP_NAME =
MONGO_AUTH_DB =
MONGO_URL = "mongodb://${MONGO_USER}:{$MONGO_PASSWORD}@${MONGO_HOST}:${MONGO_PORT}/${MONGO_DATABASE}?authSource=${MONGO_AUTH_DB}"
```

Whether the parameters are required depends only on which feature flags you use.
