CREATE TABLE "sessions"(
  id VARCHAR(36) UNIQUE DEFAULT uuid_generate_v4() NOT NULL,
  "user_id" VARCHAR(36) NOT NULL,
  username VARCHAR(32) NOT NULL,
  email VARCHAR(255) NOT NULL,
  phone VARCHAR(32),
  "role" VARCHAR(32) NOT NULL,
  csrf VARCHAR(255) UNIQUE NOT NULL,
  oauth_token VARCHAR(255) UNIQUE,
  "auth_type" VARCHAR(32) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '1 HOUR',
  CONSTRAINT pk_sessions PRIMARY KEY (id),
  CONSTRAINT fk_sessions_user_id FOREIGN KEY ("user_id") REFERENCES users(id) ON DELETE CASCADE
);
SELECT diesel_manage_updated_at('sessions');
CREATE INDEX IF NOT EXISTS sessions_user_id ON "sessions" USING BTREE(user_id);
CREATE INDEX IF NOT EXISTS sessions_btree_created_at ON "sessions" USING BTREE("created_at");
CREATE INDEX IF NOT EXISTS sessions_btree_updated_at ON "sessions" USING BTREE("updated_at");
CREATE INDEX IF NOT EXISTS sessions_btree_expires_at ON "sessions" USING BTREE("expires_at");