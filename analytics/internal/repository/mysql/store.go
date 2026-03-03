package mysql

import (
	"context"
	"database/sql"
	"time"

	"github.com/Devin-Yeung/wormhole/analytics/internal/domain"
	"github.com/Devin-Yeung/wormhole/analytics/internal/repository"
)

// type guard
var _ repository.AnalyticsRepository = (*AnalyticsStore)(nil)

type AnalyticsStore struct {
	db *sql.DB
}

func NewAnalyticsStore(db *sql.DB) *AnalyticsStore {
	return &AnalyticsStore{
		db: db,
	}
}

func (a AnalyticsStore) RecordRedirect(ctx context.Context, event *domain.RedirectEvent) error {
	//TODO implement me
	panic("implement me")
}

func (a AnalyticsStore) GetClickCount(ctx context.Context, shortCode string, since time.Time) (int64, error) {
	//TODO implement me
	panic("implement me")
}

func (a AnalyticsStore) GetTimeSeries(ctx context.Context, shortCode string, granularity domain.TimeGranularity, from, to time.Time) (*domain.TimeSeries, error) {
	//TODO implement me
	panic("implement me")
}

func (a AnalyticsStore) GetVisitorBreakdown(ctx context.Context, shortCode string, since time.Time, topN int) (*domain.VisitorBreakdown, error) {
	//TODO implement me
	panic("implement me")
}
