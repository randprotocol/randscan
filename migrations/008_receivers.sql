-- 008_receivers.sql: the receiver registry (fullnode spec 2026-09-17 §5, §8). One row per
-- published version; the current record is the highest version per id.
CREATE TABLE receivers (
    id          TEXT    NOT NULL,   -- the rand1… address
    version     INTEGER NOT NULL,
    pk          TEXT    NOT NULL,   -- hex
    kem_ek      TEXT    NOT NULL,   -- hex, 2368 chars
    signing_key TEXT    NOT NULL,   -- hex
    signature   TEXT    NOT NULL,   -- hex
    tx_hash     TEXT    NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    height      BIGINT  NOT NULL,
    PRIMARY KEY (id, version)
);
CREATE INDEX receivers_current ON receivers (id, version DESC);
