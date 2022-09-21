CREATE TABLE test_users (
  id SERIAL,
  "username" VARCHAR(2048) NOT NULL,
  "password" VARCHAR(255) NOT NULL,
  PRIMARY KEY (id)
);