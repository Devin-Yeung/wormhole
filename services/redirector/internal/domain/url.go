// Package domain defines the core value objects for the redirector service.
package domain

import "time"

// UrlRecord is a resolved URL entry returned for a redirect.
type UrlRecord struct {
	ShortCode   string
	OriginalURL string
	ExpireAt    *time.Time
}

// IsExpired returns true if the record has a non-nil expiration in the past.
func (r *UrlRecord) IsExpired() bool {
	if r.ExpireAt == nil {
		return false
	}
	return r.ExpireAt.Before(time.Now())
}
