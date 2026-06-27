package main

import (
	"context"
	"log/slog"
	"os"
	"os/signal"
	"syscall"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	slog.Info("wormhole redirector service starting")

	// TODO: init OTel (see internal/telemetry)
	// TODO: connect to MySQL read-replica (see internal/repository/mysql)
	// TODO: connect to Redis cache (see internal/repository/cached)
	// TODO: start gRPC server exposing RedirectorService

	<-ctx.Done()
	slog.Info("shutting down")
}
