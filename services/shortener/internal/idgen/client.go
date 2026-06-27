// Package idgen wraps the tinyflake gRPC service for ID generation.
package idgen

// TODO: implement a TinyflakeClient that dials the tinyflake gRPC service
// (proto/tinyflake/v1/tinyflake.proto) and returns uint64 IDs.
// The generated gRPC stubs go into gen/ via buf generate.
//
// Example usage:
//   client, err := idgen.Dial(ctx, "tinyflake:50053")
//   id, err := client.NextId(ctx)
