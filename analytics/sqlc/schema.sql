-- ============================================================
-- dim_visitors: Visitor profile
-- ============================================================
CREATE TABLE IF NOT EXISTS dim_visitors
(
    visitor_key    INT          NOT NULL AUTO_INCREMENT,
    visitor_fp     BINARY(32)   NOT NULL COMMENT 'Fingerprint: SHA256 hash of IP + User-Agent',
    ip_address     VARCHAR(45)  NOT NULL COMMENT 'Supports both IPv4 and IPv6',
    user_agent     VARCHAR(512) NULL COMMENT 'Raw User-Agent string',
    browser_family VARCHAR(64)  NULL COMMENT 'Parsed browser family, e.g. Chrome / Firefox',
    os_family      VARCHAR(64)  NULL COMMENT 'Parsed OS family, e.g. Windows / iOS',

    CONSTRAINT pk_dim_visitors PRIMARY KEY (visitor_key),
    CONSTRAINT uq_dim_visitors_fp UNIQUE (visitor_fp),

    INDEX idx_dim_visitors_ip (ip_address),
    INDEX idx_dim_visitors_browser (browser_family),
    INDEX idx_dim_visitors_os (os_family)
)
    ENGINE = InnoDB
    DEFAULT CHARSET = utf8mb4
    COMMENT = 'Visitor dimension: deduplicated profiles built from IP + User-Agent';

-- ============================================================
-- fact_clicks: Clickstream fact table
-- - event_id is UUIDv7 (time-ordered) stored as BINARY(16)
-- - clicked_at_ms remains source of truth
-- - clicked_at is generated for convenient range filters
-- ============================================================
CREATE TABLE IF NOT EXISTS fact_clicks
(
    event_id      BINARY(16) NOT NULL COMMENT 'UUIDv7 stored as 16 bytes',
    -- use INT not VARCHAR to save index space and improve join performance
    date_key      INT        NOT NULL COMMENT 'FK -> dim_date (YYYYMMDDHH)',
    url_key       INT        NOT NULL COMMENT 'FK -> dim_urls',
    visitor_key   INT        NOT NULL COMMENT 'FK -> dim_visitors',
    clicked_at_ms BIGINT     NOT NULL COMMENT 'Unix epoch ms; source of truth',
    -- clicked_at is generated for easier date range queries; not source of truth
    clicked_at    DATETIME(3) GENERATED ALWAYS AS (FROM_UNIXTIME(clicked_at_ms / 1000.0)) STORED,
    -- referer has high cardinality and is not commonly queried, so we store it as TEXT without indexing
    referer_url   TEXT       NULL COMMENT 'Raw HTTP Referer',

    CONSTRAINT pk_fact_clicks PRIMARY KEY (event_id),

    CONSTRAINT fk_fact_clicks_date FOREIGN KEY (date_key) REFERENCES dim_visitors (visitor_key),

    -- analytics / common access paths
    INDEX idx_fact_clicks_clicked_at (clicked_at),
    INDEX idx_fact_clicks_url_date (url_key, date_key),
    INDEX idx_fact_clicks_date_url (date_key, url_key),
    INDEX idx_fact_clicks_visitor_date (visitor_key, date_key),
    INDEX idx_fact_clicks_date (date_key),
    INDEX idx_fact_clicks_url (url_key),
    INDEX idx_fact_clicks_visitor (visitor_key)
)
    ENGINE = InnoDB
    DEFAULT CHARSET = utf8mb4
    COMMENT = 'Fact: one row per click; UUIDv7 PK keeps inserts mostly ordered';
