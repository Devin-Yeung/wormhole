package mysql

import (
	"context"
	"fmt"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/trace"
)

const tracerName = "wormhole/analytics/mysql"
const meterName = "wormhole/analytics/mysql"

// recordRedirectTxDuration measures wall-clock time from BeginTx to Commit (or rollback on
// error).  Buckets are sized around typical MySQL LAN round-trip latencies so
// that p50/p95/p99 are each in a separate bucket.
var recordRedirectTxDuration metric.Float64Histogram

func init() {
	var err error
	recordRedirectTxDuration, err = otel.Meter(meterName).Float64Histogram(
		"analytics.db.record_redirect.tx_duration_seconds",
		metric.WithDescription("Wall-clock duration of the RecordRedirect DB transaction from BeginTx to Commit"),
		metric.WithUnit("s"),
		metric.WithExplicitBucketBoundaries(
			0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.000,
		),
	)
	if err != nil {
		// Histogram creation only fails when the SDK is misconfigured.
		// Panic here to surface the misconfiguration at startup rather than
		// silently dropping metrics.
		panic(fmt.Sprintf("create tx_duration histogram: %v", err))
	}
}

// withSpan is a helper that wraps a function in a trace span, recording any error
func withSpan(
	ctx context.Context,
	spanName string,
	fn func(context.Context) error,
	opts ...trace.SpanStartOption,
) error {
	tracer := otel.Tracer(tracerName)
	ctx, span := tracer.Start(ctx, spanName, opts...)
	defer span.End()
	if err := fn(ctx); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return err
	}
	span.SetStatus(codes.Ok, "")
	return nil
}

// withSpanT is a generic version of withSpan that supports any return type
func withSpanT[T any](
	ctx context.Context,
	spanName string,
	fn func(context.Context) (T, error),
	opts ...trace.SpanStartOption,
) (T, error) {
	tracer := otel.Tracer(tracerName)
	ctx, span := tracer.Start(ctx, spanName, opts...)
	defer span.End()
	ret, err := fn(ctx)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return ret, err
	}
	span.SetStatus(codes.Ok, "")
	return ret, nil
}
