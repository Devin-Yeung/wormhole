# Gateway REST API Design (v1 Draft)

## Scope Based on Current Repository State

This design intentionally matches the capabilities that already exist in the workspace:

- URL creation with optional `custom_alias` and optional expiration.
- URL resolution (returns not found when code is missing or expired).
- URL deletion.
- Basic health check.

The current domain/services do **not** yet provide list/search/update/analytics features, so those are out of scope for
v1.

## API Surface

| Method   | Path                    | Purpose                    | Backing service       |
|----------|-------------------------|----------------------------|-----------------------|
| `GET`    | `/health`               | Liveness/readiness check   | gateway only          |
| `POST`   | `/v1/urls`              | Create a short URL         | `Shortener::shorten`  |
| `GET`    | `/v1/urls/{short_code}` | Get URL metadata (JSON)    | `Redirector::resolve` |
| `DELETE` | `/v1/urls/{short_code}` | Delete a short URL mapping | `Shortener::delete`   |
| `GET`    | `/{short_code}`         | Public redirect endpoint   | `Redirector::resolve` |

## Resource Model

### Create URL request

```json
{
  "original_url": "https://example.com/some/path",
  "custom_alias": "docs-home",
  "expire_at": "2026-12-31T23:59:59Z"
}
```

- `original_url` (required): must be `http://` or `https://`.
- `custom_alias` (optional): 3-32 chars, `[a-zA-Z0-9_-]`.
- `expire_at` (optional): RFC3339 UTC timestamp string.

### URL response

```json
{
  "short_code": "docs-home",
  "short_url": "https://worm.hole/docs-home",
  "original_url": "https://example.com/some/path",
  "expire_at": "2026-12-31T23:59:59Z"
}
```

### Error response

```json
{
  "error": {
    "code": "alias_conflict",
    "message": "short code already exists"
  }
}
```

## Endpoint Details

### `GET /health`

- `200 OK`

```json
{
  "status": "ok"
}
```

### `POST /v1/urls`

- `201 Created` on success.
- `Location: /v1/urls/{short_code}` header should be set.

Response body uses the URL response schema.

### `GET /v1/urls/{short_code}`

- Returns JSON metadata for API clients.
- `200 OK` with URL response schema.
- `404 Not Found` if code does not exist or has expired.

### `DELETE /v1/urls/{short_code}`

- `204 No Content` when delete succeeds.
- `404 Not Found` when the code does not exist.

### `GET /{short_code}`

- Browser-friendly redirect endpoint.
- `307 Temporary Redirect` with `Location: {original_url}`.
- `404 Not Found` if code does not exist or has expired.

`307` is preferred in v1 to avoid accidental method rewriting assumptions and to keep semantics safe while
expiration/deletion are dynamic.

## HTTP Status Mapping

| Condition                              | Status | Error code             |
|----------------------------------------|--------|------------------------|
| Invalid JSON / invalid field format    | `400`  | `invalid_request`      |
| Invalid URL                            | `400`  | `invalid_url`          |
| Invalid short code format              | `400`  | `invalid_short_code`   |
| Alias already exists                   | `409`  | `alias_conflict`       |
| Short code not found or expired        | `404`  | `short_code_not_found` |
| Storage unavailable                    | `503`  | `storage_unavailable`  |
| Storage timeout                        | `504`  | `storage_timeout`      |
| Unknown storage/cache/internal failure | `500`  | `internal_error`       |

## Notes and Constraints

- Not-found and expired are intentionally collapsed into one `404` result.
- URL expiration is checked at resolve/read time.
- API does not promise alias reuse after delete across all backends.
    - MySQL uses soft delete and keeps uniqueness history.
    - In-memory backend currently allows reuse after delete.
- This v1 contract is aligned with existing shortener/redirector/storage traits.

## Suggested Implementation Order in `wormhole-gateway`

1. Finalize `AppError` with the status mapping table above.
2. Implement `create_url_handler` by translating REST request into `ShortenParams`.
3. Implement `get_url_handler` and `GET /{short_code}` using `Redirector::resolve`.
4. Implement `delete_url_handler` with `204`/`404` behavior.
5. Add integration tests for happy path + conflict + expired/not-found.
