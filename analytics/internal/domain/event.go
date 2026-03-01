package domain

import (
	"net"
	"time"

	"github.com/Devin-Yeung/wormhole/analytics/pb/v1"
	"github.com/google/uuid"
)

// RedirectEvent is the canonical domain representation of a click.
// Converted from proto UrlRedirectedEvent at the consumer boundary,
// so the rest of the service has no dependency on protobuf.
type RedirectEvent struct {
	EventID   uuid.UUID
	ShortCode string
	ClickedAt time.Time
	VisitorIP net.IP
	UserAgent string
	Referer   string // empty string when absent
}

func (e *RedirectEvent) toProto() *pb.UrlRedirectedEvent {
	return &pb.UrlRedirectedEvent{
		EventId:     e.EventID.String(),
		ShortCode:   e.ShortCode,
		ClickedAtMs: e.ClickedAt.UnixMilli(),
		VisitorIp:   e.VisitorIP.String(),
		UserAgent:   e.UserAgent,
		Referer:     e.Referer,
	}
}
