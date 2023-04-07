# Hextacy Drivers

Contains drivers for communicating with database servers and more. DB drivers are intended to be used with the `RepositoryAccess` traits.
They provide basic functionality for building connection pools and establishing direct connections.

## Setup

The simplest way to get the necessary params for the drivers is to create `.env` file in the project root directory:

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

and use those to create the pools.
