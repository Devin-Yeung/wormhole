package domain

import (
	"net"
	"time"
)

// RedirectEvent is the canonical domain representation of a click.
// Converted from proto UrlRedirectedEvent at the consumer boundary,
// so the rest of the service has no dependency on protobuf.
type RedirectEvent struct {
	EventID   string
	ShortCode string
	ClickedAt time.Time
	VisitorIP net.IP
	UserAgent string
	Referer   string // empty string when absent
}
