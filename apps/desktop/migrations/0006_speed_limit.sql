-- D05: global bandwidth limiter. Zero means unlimited.
INSERT INTO settings (key, value, updated_at)
VALUES ('global_speed_limit_bps', '0', CURRENT_TIMESTAMP)
ON CONFLICT(key) DO NOTHING;
