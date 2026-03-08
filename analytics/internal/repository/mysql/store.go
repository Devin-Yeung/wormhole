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

// parseUserAgent extracts browser and OS family from a User-Agent string.
// TODO: implement proper parsing with a UA parser library.
func parseUserAgent(ua string) (browserFamily, osFamily string) {
	// Placeholder: return empty strings until a UA parser is integrated
	return "", ""
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
