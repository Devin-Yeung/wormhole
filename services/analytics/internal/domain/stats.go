package domain

import "time"

type TimeGranularity int

const (
	Hourly TimeGranularity = iota
	Daily
)

type TimeSeriesPoint struct {
	Timestamp time.Time
	Count     int64
}

type TimeSeries struct {
	ShortCode   string
	Granularity TimeGranularity
	Points      []TimeSeriesPoint
}

// VisitorBreakdown holds top-N entries per visitor dimension.
type VisitorBreakdown struct {
	ShortCode   string
	TopIPs      []DimensionStat
	TopUAs      []DimensionStat
	TopReferers []DimensionStat
}

type DimensionStat struct {
	Value string // IP address, UA string, or Referer URL
	Count int64
}
