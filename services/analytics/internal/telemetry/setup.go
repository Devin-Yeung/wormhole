// Package telemetry initialises the OpenTelemetry SDK for the analytics
// service.  It wires together a TracerProvider and a MeterProvider and
// registers them as the global OTel providers so that instrumented libraries
// can call otel.Tracer / otel.Meter without needing to thread a provider
// through every call site.
//
// Exporter selection is driven by the standard OTEL_EXPORTER_OTLP_ENDPOINT
// environment variable:
//   - When the variable is set, signals are exported via OTLP/gRPC to the
//     configured collector (Jaeger, Grafana Tempo, etc.).
//   - When the variable is absent, both signals fall back to a human-readable
//     stdout exporter, which is convenient for local development.
//
// Callers must invoke the returned shutdown function before the process exits
// to flush any buffered telemetry.
package telemetry

import (
	"context"
	"fmt"
	"os"
	"time"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlpmetric/otlpmetricgrpc"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/exporters/stdout/stdoutmetric"
	"go.opentelemetry.io/otel/exporters/stdout/stdouttrace"
	"go.opentelemetry.io/otel/propagation"
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.26.0"
)

const serviceName = "wormhole-analytics"

// Setup initialises both the TracerProvider and the MeterProvider, registers
// them as OTel globals, and returns a shutdown function that flushes and stops
// both providers.  The shutdown function must be called before the process
// exits; pass a context with an appropriate deadline to bound the flush time.
func Setup(ctx context.Context) (shutdown func(context.Context) error, err error) {
	// Build a resource that identifies this service in the backend UI.
	res, err := resource.New(ctx,
		resource.WithAttributes(
			semconv.ServiceName(serviceName),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("create otel resource: %w", err)
	}

	// Collect individual shutdown functions so we can call them all even if
	// one fails, without shadowing the first error.
	var shutdowns []func(context.Context) error
	addShutdown := func(fn func(context.Context) error) { shutdowns = append(shutdowns, fn) }

	// --- Tracer provider --------------------------------------------------

	tp, err := newTracerProvider(ctx, res)
	if err != nil {
		return nil, fmt.Errorf("create tracer provider: %w", err)
	}
	otel.SetTracerProvider(tp)
	addShutdown(tp.Shutdown)

	// W3C TraceContext + Baggage propagation is the de-facto standard for
	// HTTP/gRPC context propagation.
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
		propagation.TraceContext{},
		propagation.Baggage{},
	))

	// --- Meter provider ---------------------------------------------------

	mp, err := newMeterProvider(ctx, res)
	if err != nil {
		return nil, fmt.Errorf("create meter provider: %w", err)
	}
	otel.SetMeterProvider(mp)
	addShutdown(mp.Shutdown)

	// Return a single shutdown function that flushes both providers.
	shutdown = func(ctx context.Context) error {
		var firstErr error
		for _, fn := range shutdowns {
			if err := fn(ctx); err != nil && firstErr == nil {
				firstErr = err
			}
		}
		return firstErr
	}
	return shutdown, nil
}

// newTracerProvider builds a BatchSpanProcessor-backed TracerProvider.
// It uses OTLP/gRPC when OTEL_EXPORTER_OTLP_ENDPOINT is set, stdout otherwise.
func newTracerProvider(ctx context.Context, res *resource.Resource) (*sdktrace.TracerProvider, error) {
	var (
		exp sdktrace.SpanExporter
		err error
	)

	if os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT") != "" {
		exp, err = otlptracegrpc.New(ctx)
	} else {
		// Pretty-printed stdout is useful during local development.
		exp, err = stdouttrace.New(stdouttrace.WithPrettyPrint())
	}
	if err != nil {
		return nil, err
	}

	return sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exp),
		sdktrace.WithResource(res),
		// Sample every trace in development; override with
		// OTEL_TRACES_SAMPLER=parentbased_traceidratio and
		// OTEL_TRACES_SAMPLER_ARG=0.1 in production.
	), nil
}

// newMeterProvider builds a PeriodicReader-backed MeterProvider.
// It uses OTLP/gRPC when OTEL_EXPORTER_OTLP_ENDPOINT is set, stdout otherwise.
func newMeterProvider(ctx context.Context, res *resource.Resource) (*sdkmetric.MeterProvider, error) {
	var reader sdkmetric.Reader

	if os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT") != "" {
		exp, err := otlpmetricgrpc.New(ctx)
		if err != nil {
			return nil, err
		}
		reader = sdkmetric.NewPeriodicReader(exp, sdkmetric.WithInterval(15*time.Second))
	} else {
		exp, err := stdoutmetric.New()
		if err != nil {
			return nil, err
		}
		reader = sdkmetric.NewPeriodicReader(exp, sdkmetric.WithInterval(15*time.Second))
	}

	return sdkmetric.NewMeterProvider(
		sdkmetric.WithReader(reader),
		sdkmetric.WithResource(res),
	), nil
}
