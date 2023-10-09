CREATE TABLE sessions(
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  csrf uuid UNIQUE NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  CONSTRAINT fk_sessions_user_id FOREIGN KEY ("user_id") REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS sessions_user_id ON "sessions" USING BTREE(user_id);