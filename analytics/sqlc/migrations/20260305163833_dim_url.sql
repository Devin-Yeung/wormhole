-- +goose Up
-- +goose StatementBegin
CREATE TABLE IF NOT EXISTS dim_urls
(
    url_key    INT         NOT NULL AUTO_INCREMENT,
    short_code VARCHAR(32) NOT NULL COMMENT 'The short code (unique)',

    CONSTRAINT pk_dim_urls PRIMARY KEY (url_key),
    CONSTRAINT uq_dim_urls_short_code UNIQUE (short_code)
)
    ENGINE = InnoDB
    DEFAULT CHARSET = utf8mb4
    COMMENT = 'URL dimension: maps short_code to url_key for analytics';
-- +goose StatementEnd

-- +goose StatementBegin
ALTER TABLE fact_clicks
    ADD COLUMN url_key INT NOT NULL AFTER visitor_key;
-- +goose StatementEnd

-- +goose StatementBegin
ALTER TABLE fact_clicks
    ADD CONSTRAINT fk_fact_clicks_url FOREIGN KEY (url_key) REFERENCES dim_urls (url_key);
-- +goose StatementEnd

-- +goose StatementBegin
ALTER TABLE fact_clicks
    ADD INDEX idx_fact_clicks_url (url_key);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE fact_clicks
    DROP FOREIGN KEY fk_fact_clicks_url;
-- +goose StatementEnd

-- +goose StatementBegin
ALTER TABLE fact_clicks
    DROP COLUMN url_key;
-- +goose StatementEnd

-- +goose StatementBegin
DROP TABLE IF EXISTS dim_urls;
-- +goose StatementEnd
