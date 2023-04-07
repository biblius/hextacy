CREATE TABLE "oauth"(
  id VARCHAR(36) UNIQUE DEFAULT uuid_generate_v4() NOT NULL,
  "user_id" VARCHAR(36) NOT NULL,
  access_token VARCHAR(255) NOT NULL,
  refresh_token VARCHAR(255),
  "provider" VARCHAR(255) NOT NULL,
  account_id VARCHAR(255) NOT NULL,
  scope VARCHAR(2048) NOT NULL,
  revoked BOOLEAN NOT NULL DEFAULT FALSE,
  expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT pk_oauth_meta PRIMARY KEY (id),
  CONSTRAINT fk_oauth_meta_user_id FOREIGN KEY ("user_id") REFERENCES users(id) ON DELETE CASCADE
);
SELECT diesel_manage_updated_at('oauth');