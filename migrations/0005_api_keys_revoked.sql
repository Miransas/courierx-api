ALTER TABLE api_keys
    ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_api_keys_workspace_id
    ON api_keys (workspace_id);

CREATE INDEX IF NOT EXISTS idx_api_keys_revoked
    ON api_keys (revoked_at)
    WHERE revoked_at IS NOT NULL;
