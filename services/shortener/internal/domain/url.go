// Package domain defines the core value objects for the shortener service.
package domain

import "time"

// ShortCode is a validated short code identifier for a shortened URL.
// Codes must be 3-32 characters, alphanumeric with hyphens and underscores.
type ShortCode struct {
	value string
}

// NewShortCode creates a validated ShortCode. Returns an error if the value
// is outside the allowed character set or length range.
func NewShortCode(s string) (ShortCode, error) {
	if err := validateShortCode(s); err != nil {
		return ShortCode{}, err
	}
	return ShortCode{value: s}, nil
}

func (c ShortCode) String() string { return c.value }

// UrlRecord is a stored URL entry.
type UrlRecord struct {
	ShortCode   ShortCode
	OriginalURL string
	ExpireAt    *time.Time
}

// ShortenParams carries the inputs for a shorten operation.
type ShortenParams struct {
	OriginalURL string
	CustomAlias *string    // nil means auto-generate
	ExpireAt    *time.Time // nil means never
}

func validateShortCode(s string) error {
	const minLen, maxLen = 3, 32
	if len(s) < minLen || len(s) > maxLen {
		return &ValidationError{Field: "short_code", Message: "length must be 3-32 characters"}
	}
	for _, c := range s {
		if !isAllowed(c) {
			return &ValidationError{Field: "short_code", Message: "only alphanumeric, hyphen, underscore allowed"}
		}
	}
	return nil
}

func isAllowed(c rune) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
		(c >= '0' && c <= '9') || c == '-' || c == '_'
}

// ValidationError is returned when a domain value fails its invariants.
type ValidationError struct {
	Field   string
	Message string
}

func (e *ValidationError) Error() string {
	return e.Field + ": " + e.Message
}
