// Package service defines the RedirectorService interface.
package service

import (
	"context"

	"github.com/Devin-Yeung/wormhole/services/redirector/internal/domain"
)

// RedirectorService is the primary port for URL redirection operations.
type RedirectorService interface {
	// Resolve returns the UrlRecord for the given short code.
	// Returns nil without error if the code does not exist.
	Resolve(ctx context.Context, shortCode string) (*domain.UrlRecord, error)
}

// TODO: implement RedirectorServiceImpl backed by repository + cache
