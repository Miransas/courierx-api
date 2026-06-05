ALTER TABLE emails ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
CREATE INDEX idx_emails_updated_at ON emails(updated_at);
