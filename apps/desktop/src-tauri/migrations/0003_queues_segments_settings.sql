CREATE TABLE IF NOT EXISTS segments (
    id TEXT PRIMARY KEY NOT NULL,
    download_id TEXT NOT NULL REFERENCES downloads(id) ON DELETE CASCADE,
    segment_index INTEGER NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL,
    downloaded_bytes INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'queued',
    UNIQUE(download_id, segment_index)
);

CREATE TABLE IF NOT EXISTS queues (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    max_concurrent INTEGER NOT NULL DEFAULT 3,
    auto_start INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_segments_download ON segments(download_id);
CREATE INDEX IF NOT EXISTS idx_segments_status ON segments(status);
CREATE INDEX IF NOT EXISTS idx_queues_priority ON queues(priority DESC);

INSERT OR IGNORE INTO settings (key, value) VALUES
    ('max_concurrent_downloads', '3'),
    ('theme', 'midnight'),
    ('notifications_enabled', '1');
