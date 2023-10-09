CREATE VIEW user_sessions AS (
  SELECT
    s.id as session_id,
    u.id as user_id,
    u.username,
    s.csrf,
    s.created_at,
    s.updated_at,
    s.expires_at
  FROM sessions s
  INNER JOIN users u ON s.user_id = u.id);
