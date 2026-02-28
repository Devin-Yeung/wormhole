package repository

import (
	"context"
	"time"

	"github.com/Devin-Yeung/wormhole/analytics/internal/domain"
)

// AnalyticsRepository is the storage contract for the analytics service.
// Implementations may be backed by ClickHouse, TimescaleDB, MySQL, etc.
type AnalyticsRepository interface {
	// --- Write path ---

	// RecordRedirect persists a single redirect event.
	RecordRedirect(ctx context.Context, event *domain.RedirectEvent) error

	// --- Query path ---

	// GetClickCount returns total clicks for shortCode in [since, now).
	GetClickCount(ctx context.Context, shortCode string, since time.Time) (int64, error)

	// GetTimeSeries returns aggregated click counts bucketed by granularity
	// in the half-open interval [from, to).
	GetTimeSeries(
		ctx context.Context,
		shortCode string,
		granularity domain.TimeGranularity,
		from, to time.Time,
	) (*domain.TimeSeries, error)

	// GetVisitorBreakdown returns the top-topN entries for each visitor
	// dimension (IP, UA, Referer) for shortCode since the given time.
	GetVisitorBreakdown(
		ctx context.Context,
		shortCode string,
		since time.Time,
		topN int,
	) (*domain.VisitorBreakdown, error)
}
