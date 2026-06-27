package service

import (
	"context"
	"time"

	"github.com/Devin-Yeung/wormhole/analytics/internal/domain"
)

// Query structs encapsulate all parameters for each query type,
// making it easy to add optional fields (e.g. timezone) later
// without breaking existing call sites.

type ClickCountQuery struct {
	ShortCode string
	Since     time.Time
}

type TimeSeriesQuery struct {
	ShortCode   string
	Granularity domain.TimeGranularity
	From        time.Time
	To          time.Time
}

type VisitorQuery struct {
	ShortCode string
	Since     time.Time
	TopN      int // defaults to 10 if zero
}

// AnalyticsService is the application-layer facade consumed by API handlers.
type AnalyticsService interface {
	GetClickCount(ctx context.Context, q ClickCountQuery) (int64, error)
	GetTimeSeries(ctx context.Context, q TimeSeriesQuery) (*domain.TimeSeries, error)
	GetVisitorBreakdown(ctx context.Context, q VisitorQuery) (*domain.VisitorBreakdown, error)
}
