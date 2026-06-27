// Package repository defines the storage port for the redirector service.
package repository

import (
	"context"

	"github.com/Devin-Yeung/wormhole/services/redirector/internal/domain"
)

// ReadRepository is the read-only storage port for URL lookups.
type ReadRepository interface {
	// Get retrieves a URL record by short code. Returns nil without error if
	// the code does not exist or has expired.
	Get(ctx context.Context, shortCode string) (*domain.UrlRecord, error)
}
