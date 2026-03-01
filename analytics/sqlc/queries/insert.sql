-- name: InsertVisitor :exec
-- Inserts a new visitor or returns the existing visitor_key if visitor_fp already exists.
-- Uses ON DUPLICATE KEY UPDATE to handle the unique constraint on visitor_fp.
-- Note: caller should query for visitor_key using visitor_fp after this call.
INSERT INTO dim_visitors (visitor_fp, ip_address, user_agent, browser_family, os_family)
VALUES (?, ?, ?, ?, ?)
ON DUPLICATE KEY UPDATE visitor_key = LAST_INSERT_ID(visitor_key);

-- name: InsertClick :exec
-- Inserts a click fact record.
-- caller is responsible for providing valid date_key and url_key values.
INSERT INTO fact_clicks (event_id, date_key, url_key, visitor_key, clicked_at_ms, referer_url)
VALUES (?, ?, ?, ?, ?, ?);
