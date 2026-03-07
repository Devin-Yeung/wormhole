package mysql

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"errors"
	"time"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/metric"

	"github.com/Devin-Yeung/wormhole/analytics/internal/domain"
	"github.com/Devin-Yeung/wormhole/analytics/internal/repository"
	. "github.com/Devin-Yeung/wormhole/analytics/internal/repository/mysql/internal/sqlc"
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
//
// Observability:
//   - A parent span covers the full operation including BeginTx and Commit.
//   - Each DB step has its own child span so slow queries are pinpointed in
//     the trace waterfall (e.g. Jaeger, Tempo).
//   - recordRedirectTxDuration histogram records end-to-end latency with a "status" attribute
//     ("ok" | "error") for alerting and SLO dashboards.
func (a *AnalyticsStore) RecordRedirect(ctx context.Context, event *domain.RedirectEvent) error {
	ctx, span := otel.Tracer(tracerName).Start(ctx, "AnalyticsStore.RecordRedirect")
	defer span.End()

	txStart := time.Now()

	// statusAttr is set to "error" before any fallible operation and reset to
	// "ok" only on successful commit, so the histogram always reflects the
	// true outcome even if we return early.
	statusAttr := attribute.String("status", "error")
	defer func() {
		recordRedirectTxDuration.Record(ctx, time.Since(txStart).Seconds(),
			metric.WithAttributes(statusAttr),
		)
	}()

	// Compute visitor fingerprint: SHA256(IP + UserAgent)
	fp := sha256.Sum256([]byte(event.VisitorIP.String() + event.UserAgent))
	visitorFp := fp[:]

	// Parse browser and OS families from User-Agent
	// TODO: integrate a UA parser library (e.g., github.com/mssola/user_agent)
	browserFamily, osFamily := parseUserAgent(event.UserAgent)

	tx, err := a.db.BeginTx(ctx, nil)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "BeginTx failed")
		return err
	}

	// always defer the rollback to make sure the transaction is cleaned up in case of an error
	// It's safe because if the transaction has already been committed, the rollback will return sql.ErrTxDone, which we can ignore.
	defer func(tx *sql.Tx) {
		err := tx.Rollback()
		if err != nil && !errors.Is(err, sql.ErrTxDone) {
			span.RecordError(err)
		}
	}(tx)

	q := New(tx).WithTx(tx)

	// 1. Insert or get URL key for short_code
	if err := withSpan(ctx, "db.InsertUrl", func(ctx context.Context) error {
		return q.InsertUrl(ctx, event.ShortCode)
	}); err != nil {
		span.SetStatus(codes.Error, "InsertUrl failed")
		return err
	}
	var urlKey int32

	if err := withSpan(ctx, "db.GetUrlKey", func(ctx context.Context) error {
		var e error
		urlKey, e = q.GetUrlKey(ctx, event.ShortCode)
		return e
	}); err != nil {
		span.SetStatus(codes.Error, "GetUrlKey failed")
		return err
	}

	// 2. Insert or get visitor key for visitor fingerprint
	if err := withSpan(ctx, "db.InsertVisitor", func(ctx context.Context) error {
		return q.InsertVisitor(ctx, InsertVisitorParams{
			VisitorFp:     visitorFp,
			IpAddress:     event.VisitorIP.String(),
			UserAgent:     sql.NullString{String: event.UserAgent, Valid: event.UserAgent != ""},
			BrowserFamily: sql.NullString{String: browserFamily, Valid: browserFamily != ""},
			OsFamily:      sql.NullString{String: osFamily, Valid: osFamily != ""},
		})
	}); err != nil {
		span.SetStatus(codes.Error, "InsertVisitor failed")
		return err
	}

	var visitorKey int32
	if visitorKey, err = withSpanT(ctx, "db.GetVisitorKey", func(ctx context.Context) (int32, error) {
		return q.GetVisitorKey(ctx, visitorFp)
	}); err != nil {
		span.SetStatus(codes.Error, "GetVisitorKey failed")
		return err
	}

	// 3. Insert the click fact record
	eventID := event.EventID[:] // UUID is 16 bytes
	if err := withSpan(ctx, "db.InsertClick", func(ctx context.Context) error {
		return q.InsertClick(ctx, InsertClickParams{
			EventID:     eventID,
			UrlKey:      urlKey,
			VisitorKey:  visitorKey,
			ClickedAtMs: event.ClickedAt.UnixMilli(),
			RefererUrl:  sql.NullString{String: event.Referer, Valid: event.Referer != ""},
		})
	}); err != nil {
		span.SetStatus(codes.Error, "InsertClick failed")
		return err
	}

	if err := tx.Commit(); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "Commit failed")
		return err
	}

	// All steps succeeded; update statusAttr so the deferred histogram
	// records "ok" rather than the default "error".
	statusAttr = attribute.String("status", "ok")
	span.SetStatus(codes.Ok, "")
	return nil
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
