-- name: InsertUrl :exec
-- Inserts a new URL or returns the existing url_key if short_code already exists.
-- Uses ON DUPLICATE KEY UPDATE to handle the unique constraint on short_code.
INSERT INTO dim_urls (short_code)
VALUES (?)
ON DUPLICATE KEY UPDATE url_key = LAST_INSERT_ID(url_key);

-- name: GetUrlKey :one
-- Gets url_key for a given short_code.
SELECT url_key FROM dim_urls WHERE short_code = ?;

-- name: InsertVisitor :exec
-- Inserts a new visitor or returns the existing visitor_key if visitor_fp already exists.
-- Uses ON DUPLICATE KEY UPDATE to handle the unique constraint on visitor_fp.
INSERT INTO dim_visitors (visitor_fp, ip_address, user_agent, browser_family, os_family)
VALUES (?, ?, ?, ?, ?)
ON DUPLICATE KEY UPDATE visitor_key = LAST_INSERT_ID(visitor_key);

-- name: GetVisitorKey :one
-- Gets visitor_key for a given visitor_fp.
SELECT visitor_key FROM dim_visitors WHERE visitor_fp = ?;

-- name: InsertClick :exec
-- Inserts a click fact record.
-- caller is responsible for providing valid url_key and visitor_key values.
INSERT INTO fact_clicks (event_id, url_key, visitor_key, clicked_at_ms, referer_url)
VALUES (?, ?, ?, ?, ?);
