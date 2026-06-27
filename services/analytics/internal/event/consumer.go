package event

import (
	"context"
	"io"

	"github.com/Devin-Yeung/wormhole/analytics/internal/domain"
)

// Consumer abstracts the Kafka subscription mechanism.
// The concrete implementation uses segmentio/kafka-go or confluent-kafka-go.
type Consumer interface {
	// Subscribe blocks until ctx is cancelled, dispatching each decoded
	// UrlRedirectedEvent to handler.
	// Offsets are committed only after Handle returns nil (at-least-once semantics).
	Subscribe(ctx context.Context, handler Handler) error
	io.Closer
}

// Handler processes a single decoded event.
// Returning a non-nil error signals the consumer to NOT commit the offset,
// triggering a retry on the next poll.
type Handler interface {
	Handle(ctx context.Context, event *domain.RedirectEvent) error
}
