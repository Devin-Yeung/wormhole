-- +goose Up
-- +goose StatementBegin
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
-- +goose StatementEnd

-- +goose StatementBegin
ALTER TABLE fact_clicks
    ADD CONSTRAINT fk_fact_clicks_visitor FOREIGN KEY (visitor_key) REFERENCES dim_visitors (visitor_key);
-- +goose StatementEnd

-- +goose Down

-- +goose StatementBegin
ALTER TABLE fact_clicks
    DROP FOREIGN KEY fk_fact_clicks_visitor;
-- +goose StatementEnd

-- +goose StatementBegin
DROP TABLE IF EXISTS dim_visitors;
-- +goose StatementEnd
