package event

import (
	"bytes"
	"fmt"
	"net"
	"time"

	"github.com/Devin-Yeung/wormhole/analytics/internal/domain"
	pb "github.com/Devin-Yeung/wormhole/analytics/pb/v1"
	"github.com/google/uuid"
)

// protoToRedirectEvent converts a wire-level UrlRedirectedEvent to the
// canonical domain type. This is the only place in the service that
// touches protobuf types; everything downstream works with domain.RedirectEvent.
func protoToRedirectEvent(e *pb.UrlRedirectedEvent) (*domain.RedirectEvent, error) {
	// EventId should be a string-encoded UUIDv7.
	reader := bytes.NewReader([]byte(e.EventId))
	eventID, err := uuid.NewV7FromReader(reader)
	if err != nil {
		return nil, fmt.Errorf("malformed event_id %q: %w", e.EventId, err)
	}

	// ClickedAtMs is milliseconds since Unix epoch. We normalise to UTC so
	// storage backends and query callers don't need to guess the timezone.
	clickedAt := time.UnixMilli(e.ClickedAtMs).UTC()

	// net.ParseIP accepts both IPv4 ("1.2.3.4") and IPv6 ("::1") notation.
	// A nil result means the field is empty or malformed; we reject the event
	// rather than silently storing a missing IP, since visitor attribution
	// is a core requirement of the analytics schema.
	ip := net.ParseIP(e.VisitorIp)
	if ip == nil {
		return nil, fmt.Errorf("event %q: invalid visitor_ip %q", e.EventId, e.VisitorIp)
	}

	return &domain.RedirectEvent{
		EventID:   eventID,
		ShortCode: e.ShortCode,
		ClickedAt: clickedAt,
		VisitorIP: ip,
		UserAgent: e.UserAgent,
		Referer:   e.Referer, // proto default ("") matches domain convention
	}, nil
}
