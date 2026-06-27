// Package service defines the ShortenerService interface.
package service

import (
	"context"

	"github.com/Devin-Yeung/wormhole/services/shortener/internal/domain"
)

// ShortenerService is the primary port for URL shortening operations.
type ShortenerService interface {
	// Shorten creates a new short URL and returns the resulting record.
	Shorten(ctx context.Context, params domain.ShortenParams) (*domain.UrlRecord, error)
	// Resolve looks up the original URL for a given short code.
	Resolve(ctx context.Context, code domain.ShortCode) (*domain.UrlRecord, error)
	// Delete removes a short URL. Returns false if the code did not exist.
	Delete(ctx context.Context, code domain.ShortCode) (bool, error)
}

// TODO: implement ShortenerServiceImpl backed by repository + idgen
