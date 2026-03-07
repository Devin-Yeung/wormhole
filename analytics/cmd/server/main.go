package main

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/Devin-Yeung/wormhole/analytics/internal/telemetry"
)

func main() {
	// Use a cancellable root context so shutdown is coordinated when we receive
	// SIGINT / SIGTERM.
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	if err := run(ctx); err != nil {
		slog.Error("fatal", "err", err)
		os.Exit(1)
	}
}

func run(ctx context.Context) error {
	// Initialise OTel before anything else so that all instrumented code (e.g.
	// the analytics store) can call otel.Tracer / otel.Meter immediately.
	// The SDK reads OTEL_EXPORTER_OTLP_ENDPOINT from the environment:
	//   - set  → export to collector via OTLP/gRPC
	//   - unset → pretty-print to stdout (convenient for local dev)
	otelShutdown, err := telemetry.Setup(ctx)
	if err != nil {
		return fmt.Errorf("setup otel: %w", err)
	}
	defer func() {
		// Give in-flight spans and metrics up to 5 seconds to flush.
		flushCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := otelShutdown(flushCtx); err != nil {
			slog.Warn("otel shutdown error", "err", err)
		}
	}()

	slog.Info("Wormhole Analytics Service started")

	// TODO: start gRPC server, Kafka consumer, etc.

	// Block until the process receives a termination signal.
	<-ctx.Done()
	slog.Info("shutting down")
	return nil
}
