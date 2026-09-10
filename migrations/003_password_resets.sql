-- Password reset tokens: the token itself is never stored, only its SHA-256 hex.
-- One row per issued link; consumed (used_at set) exactly once, expires after one hour.

CREATE TABLE password_resets (
    token_hash  CHAR(64) PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ
);

CREATE INDEX idx_password_resets_user ON password_resets(user_id);
CREATE INDEX idx_password_resets_expires ON password_resets(expires_at);
