-- +goose Up
-- +goose StatementBegin
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
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DROP TABLE IF EXISTS fact_clicks;
-- +goose StatementEnd
