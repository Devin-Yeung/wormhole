// Package repository defines the storage interface for the shortener service.
package repository

import (
	"context"

	"github.com/Devin-Yeung/wormhole/services/shortener/internal/domain"
)

// Repository is the storage port for URL records.
type Repository interface {
	// Save persists a new URL record. Returns ErrConflict if the short code
	// already exists.
	Save(ctx context.Context, record *domain.UrlRecord) error
	// Get retrieves a URL record by short code. Returns nil without error if
	// the code does not exist.
	Get(ctx context.Context, code domain.ShortCode) (*domain.UrlRecord, error)
	// Delete removes a URL record. Returns false if the code did not exist.
	Delete(ctx context.Context, code domain.ShortCode) (bool, error)
}

// ErrConflict is returned when a short code already exists.
type ErrConflict struct{ ShortCode string }

func (e *ErrConflict) Error() string {
	return "short code already exists: " + e.ShortCode
}
