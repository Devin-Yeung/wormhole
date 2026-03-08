package mysql

import (
	"context"
	"net"
	"sync/atomic"
	"testing"
	"time"

	"github.com/Devin-Yeung/wormhole/analytics/internal/domain"
	"github.com/Devin-Yeung/wormhole/analytics/internal/telemetry"
	"github.com/google/uuid"
	"github.com/stretchr/testify/require"
)

func BenchmarkRecordRedirect(b *testing.B) {
	ctx := context.Background()

	otelShutdown, err := telemetry.Setup(ctx)
	defer func() {
		// Give in-flight spans and metrics up to 5 seconds to flush.
		flushCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		_ = otelShutdown(flushCtx)
	}()

	require.NoError(b, err)

	// We spin up MySQL once per benchmark run so the reported time focuses on
	// steady-state write throughput, not container startup and migrations.
	db, shutdown := NewMysql(ctx, b)
	b.Cleanup(shutdown)

	// Keep the pool bounded so we can reason about throughput changes when
	// tuning parallelism rather than accidentally opening too many DB sessions.
	db.SetMaxOpenConns(64)
	db.SetMaxIdleConns(64)
	db.SetConnMaxLifetime(0)

	store := NewAnalyticsStore(db)
	var eventSeq uint64

	// TODO: add more traffic shapes (hot visitor, mixed short codes) once we
	// capture the first baseline from this simple "mostly-unique events" load.
	const shortCode = "bench-short-code"
	const userAgent = "benchmark-client/1.0"
	const referer = "https://benchmark.local/load-test"

	// pre-generate the data
	events := make([]*domain.RedirectEvent, b.N)

	for i := 0; i < b.N; i++ {
		events[i] = &domain.RedirectEvent{
			EventID:   uuid.Must(uuid.NewV7()),
			ShortCode: shortCode,
			ClickedAt: time.Now().UTC(),
			VisitorIP: net.IPv4(
				10,
				byte((i>>16)&0xff),
				byte((i>>8)&0xff),
				byte(i&0xff),
			).To4(),
			UserAgent: userAgent,
			Referer:   referer,
		}
	}

	b.ReportAllocs()
	b.ResetTimer()

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			seq := atomic.AddUint64(&eventSeq, 1)
			event := events[seq%uint64(len(events))]
			err := store.RecordRedirect(ctx, event)
			require.NoError(b, err)
		}
	})

	b.ReportMetric(float64(b.N)/b.Elapsed().Seconds(), "events/sec")
}
