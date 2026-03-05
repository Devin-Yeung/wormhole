package mysql

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"errors"
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

// RecordRedirect records a redirect event to the analytics store.
// It uses a transaction to ensure atomicity of:
// 1. Insert or get URL key for short_code
// 2. Insert or get visitor key for visitor fingerprint
// 3. Insert the click fact record
func (a *AnalyticsStore) RecordRedirect(ctx context.Context, event *domain.RedirectEvent) error {
	// Compute visitor fingerprint: SHA256(IP + UserAgent)
	fp := sha256.Sum256([]byte(event.VisitorIP.String() + event.UserAgent))
	visitorFp := fp[:]

	// Parse browser and OS families from User-Agent
	// TODO: integrate a UA parser library (e.g., github.com/mssola/user_agent)
	browserFamily, osFamily := parseUserAgent(event.UserAgent)

	tx, err := a.db.BeginTx(ctx, nil)
	if err != nil {
		return err
	}

	// always defer the rollback to make sure the transaction is cleaned up in case of an error
	// It's safe because if the transaction has already been committed, the rollback will return sql.ErrTxDone, which we can ignore.
	defer func(tx *sql.Tx) {
		err := tx.Rollback()
		if err != nil && !errors.Is(err, sql.ErrTxDone) {
			// Log the rollback error, but don't override the original error if there is one
		}
	}(tx)

	q := New(tx).WithTx(tx)

	// 1. Insert or get URL key for short_code
	if err := q.InsertUrl(ctx, event.ShortCode); err != nil {
		return err
	}
	urlKey, err := q.GetUrlKey(ctx, event.ShortCode)
	if err != nil {
		return err
	}

	// 2. Insert or get visitor key for visitor fingerprint
	if err := q.InsertVisitor(ctx, InsertVisitorParams{
		VisitorFp:     visitorFp,
		IpAddress:     event.VisitorIP.String(),
		UserAgent:     sql.NullString{String: event.UserAgent, Valid: event.UserAgent != ""},
		BrowserFamily: sql.NullString{String: browserFamily, Valid: browserFamily != ""},
		OsFamily:      sql.NullString{String: osFamily, Valid: osFamily != ""},
	}); err != nil {
		return err
	}
	visitorKey, err := q.GetVisitorKey(ctx, visitorFp)
	if err != nil {
		return err
	}

	// 3. Insert the click fact record
	eventID := event.EventID[:] // UUID is 16 bytes
	if err := q.InsertClick(ctx, InsertClickParams{
		EventID:     eventID,
		UrlKey:      urlKey,
		VisitorKey:  visitorKey,
		ClickedAtMs: event.ClickedAt.UnixMilli(),
		RefererUrl:  sql.NullString{String: event.Referer, Valid: event.Referer != ""},
	}); err != nil {
		return err
	}

	return tx.Commit()
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
