-- RandScan accounts: users, cookie sessions and API keys. Secrets are stored as SHA-256 hex only.

CREATE TABLE users (
    id             BIGSERIAL PRIMARY KEY,
    email          VARCHAR(254) NOT NULL,
    password_hash  TEXT NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at  TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_users_email ON users(email);

CREATE TABLE sessions (
    token_hash    CHAR(64) PRIMARY KEY,
    user_id       BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at    TIMESTAMPTZ NOT NULL,
    last_seen_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_agent    VARCHAR(256),
    ip            VARCHAR(64)
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);

CREATE TABLE api_keys (
    id             BIGSERIAL PRIMARY KEY,
    user_id        BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name           VARCHAR(64) NOT NULL,
    prefix         VARCHAR(16) NOT NULL,
    key_hash       CHAR(64) NOT NULL UNIQUE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at   TIMESTAMPTZ,
    request_count  BIGINT NOT NULL DEFAULT 0,
    revoked_at     TIMESTAMPTZ
);

CREATE INDEX idx_api_keys_user ON api_keys(user_id, created_at DESC);
