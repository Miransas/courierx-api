CREATE TABLE workspaces (
    id         UUID        PRIMARY KEY,
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE api_keys (
    id            UUID        PRIMARY KEY,
    workspace_id  UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name          TEXT        NOT NULL,
    key_prefix    TEXT        NOT NULL UNIQUE,
    key_hash      TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at  TIMESTAMPTZ
);

CREATE TABLE emails (
    id                  UUID        PRIMARY KEY,
    workspace_id        UUID        NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    api_key_id          UUID        NOT NULL REFERENCES api_keys(id)   ON DELETE RESTRICT,
    from_addr           TEXT        NOT NULL,
    to_addrs            TEXT[]      NOT NULL,
    cc_addrs            TEXT[],
    bcc_addrs           TEXT[],
    reply_to            TEXT,
    subject             TEXT        NOT NULL,
    html_body           TEXT,
    text_body           TEXT,
    headers             JSONB       NOT NULL DEFAULT '{}'::jsonb,
    status              TEXT        NOT NULL DEFAULT 'queued',
    provider            TEXT,
    provider_message_id TEXT,
    error               TEXT,
    attempts            INT         NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at             TIMESTAMPTZ
);

CREATE INDEX emails_status_created_at_idx ON emails (status, created_at);
CREATE INDEX emails_workspace_id_idx      ON emails (workspace_id);
