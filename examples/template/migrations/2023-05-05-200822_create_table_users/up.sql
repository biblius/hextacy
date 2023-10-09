CREATE TABLE users(
  id uuid PRIMARY KEY,
  username VARCHAR(32) NOT NULL,
  password VARCHAR(255) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);
