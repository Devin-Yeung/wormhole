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

	slog.Info("wormhole shortener service starting")

	// TODO: init OTel (see internal/telemetry)
	// TODO: connect to MySQL (see internal/repository/mysql)
	// TODO: connect to tinyflake gRPC (see internal/idgen)
	// TODO: start gRPC server exposing ShortenerService

	<-ctx.Done()
	slog.Info("shutting down")
}
