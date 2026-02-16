CREATE TABLE IF NOT EXISTS short_urls
(
    short_code   VARCHAR(32) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
    original_url TEXT                                              NOT NULL,
    expire_at    BIGINT                                            NULL,
    deleted_at   BIGINT                                            NULL,
    PRIMARY KEY (short_code)
) ENGINE = InnoDB;
