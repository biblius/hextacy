CREATE TABLE users(
  id VARCHAR(36) UNIQUE DEFAULT uuid_generate_v4() NOT NULL,
  email VARCHAR(255) UNIQUE NOT NULL,
  username VARCHAR(32) NOT NULL,
  first_name VARCHAR(255),
  last_name VARCHAR(255),
  "role" VARCHAR(32) NOT NULL DEFAULT 'user',
  phone VARCHAR(32),
  "password" VARCHAR(255),
  otp_secret VARCHAR(320),
  frozen BOOLEAN NOT NULL DEFAULT FALSE,
  google_id VARCHAR(255),
  github_id VARCHAR(255),
  email_verified_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT pk_users PRIMARY KEY (id)
);
-- Diesel helper to automatically adjust the 'updated_at' field
SELECT diesel_manage_updated_at('users');
CREATE INDEX IF NOT EXISTS users_email ON users USING BTREE(email);
CREATE INDEX IF NOT EXISTS users_first_name ON users USING BTREE(username);
CREATE INDEX IF NOT EXISTS users_google_id ON users USING BTREE(google_id);
CREATE INDEX IF NOT EXISTS users_github_id ON users USING BTREE(github_id);
CREATE INDEX IF NOT EXISTS users_btree_created_at ON users USING BTREE("created_at");
CREATE INDEX IF NOT EXISTS users_btree_updated_at ON users USING BTREE("updated_at");