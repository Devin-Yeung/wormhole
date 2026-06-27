// Package cached wraps a ReadRepository with a Redis-backed cache layer.
//
// The cache uses a read-through strategy: on a miss the inner repository is
// queried and the result is stored in Redis with a TTL. Negative results
// (nil) can also be cached to avoid thundering-herd against MySQL.
package cached

// TODO: implement CachedRepository that wraps repository.ReadRepository with
// a Redis cache. Mirror the layered-cache pattern from the Rust redirector.
