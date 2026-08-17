ALTER TABLE queues ADD COLUMN start_at TEXT;
ALTER TABLE queues ADD COLUMN stop_at TEXT;

CREATE INDEX IF NOT EXISTS idx_queues_schedule ON queues(start_at, stop_at);

INSERT OR IGNORE INTO settings (key, value) VALUES
    ('scheduler_enabled', '0'),
    ('scheduler_timezone', 'local');

-- Migration marker is intentionally represented by the schema itself; startup
-- applies this file once and tolerates duplicate-column errors on later runs.

