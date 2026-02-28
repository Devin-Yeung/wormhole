# Wormhole

A high-performance URL shortener service written in Rust with gRPC and HTTP APIs, multi-tier caching, and MySQL persistence.

```bash
# Create a short URL
curl -X POST https://worm.hole/v1/urls \
  -H "Content-Type: application/json" \
  -d '{"original_url": "https://example.com/very/long/path"}'

# Response:
# {
#   "short_code": "3X7k9m",
#   "short_url": "https://worm.hole/3X7k9m",
#   "original_url": "https://example.com/very/long/path"
# }

# Redirect users to the original URL
curl -v https://worm.hole/3X7k9m
# → 307 redirect to https://example.com/very/long/path
```

## Features

- **High performance**: Built in Rust with async I/O, multi-tier caching (Moka local + Redis), and efficient ID generation
- **Microservices architecture**: gRPC communication between services (shortener, redirector, gateway)
- **Production-ready**: Redis Sentinel for high availability, MySQL with migrations, health checks
- **Flexible backends**: In-memory repository for testing, Redis cache, MySQL for persistence
- Custom aliases (e.g., `worm.hole/docs` instead of random codes)
- Optional expiration times
- Soft delete with uniqueness preservation (MySQL)
- Request coalescing via single-flight cache pattern
- Bloom filter for negative caching

## Development

### Run locally

```bash
# Start Redis and MySQL containers
just dev-up
```

Wormhole consists of three services that communicate via gRPC:

```bash
# Terminal 1: Start the shortener service (creates/shortens URLs)
cargo run -p wormhole-shortener

# Terminal 2: Start the redirector service (resolves/redirects URLs)
cargo run -p wormhole-redirector

# Terminal 3: Start the gateway HTTP server
cargo run -p wormhole-gateway
```

## Deployment

Deploy all services with Docker Compose:

```bash
docker-compose up -d
```

This starts:
- **gateway** on port 8080 (HTTP API)
- **shortener** gRPC service (port 50051)
- **redirector** gRPC service (port 50052)
- **redis** for caching
- **mysql** for persistence

Configure services via environment variables (see `docker-compose.yml` for examples).

### API Examples

```bash
# Health check
curl https://worm.hole/health

# Create short URL (auto-generated code)
curl -X POST https://worm.hole/v1/urls \
  -H "Content-Type: application/json" \
  -d '{"original_url": "https://rust-lang.org"}'

# Create short URL with custom alias
curl -X POST https://worm.hole/v1/urls \
  -H "Content-Type: application/json" \
  -d '{
    "original_url": "https://docs.rs/tokio",
    "custom_alias": "tokio-docs",
    "expire_at": "2026-12-31T23:59:59Z"
  }'

# Get URL metadata
curl https://worm.hole/v1/urls/tokio-docs

# Delete short URL
curl -X DELETE https://worm.hole/v1/urls/tokio-docs
```
