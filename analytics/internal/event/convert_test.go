package event

import (
	"net"
	"testing"
	"time"

	"github.com/Devin-Yeung/wormhole/analytics/pb/v1"
	"github.com/google/uuid"
	"github.com/stretchr/testify/require"
)

func TestProtoToRedirectEvent(t *testing.T) {
	// Milliseconds for 2024-01-15 10:30:00 UTC.
	const ts int64 = 1705314600000

	// Valid UUIDv7 strings for test cases.
	uuid1 := uuid.Must(uuid.NewV7()).String()
	uuid2 := uuid.Must(uuid.NewV7()).String()
	uuid3 := uuid.Must(uuid.NewV7()).String()
	uuid4 := uuid.Must(uuid.NewV7()).String()

	tests := []struct {
		name        string
		input       *pb.UrlRedirectedEvent
		wantErr     bool
		wantIP      net.IP
		wantTime    time.Time
		wantReferer string
	}{
		{
			name: "valid IPv4 with referer",
			input: &pb.UrlRedirectedEvent{
				EventId:     uuid1,
				ShortCode:   "abc123",
				ClickedAtMs: ts,
				VisitorIp:   "203.0.113.42",
				UserAgent:   "Mozilla/5.0",
				Referer:     "https://example.com",
			},
			wantIP:      net.ParseIP("203.0.113.42"),
			wantTime:    time.UnixMilli(ts).UTC(),
			wantReferer: "https://example.com",
		},
		{
			name: "valid IPv6 without referer",
			input: &pb.UrlRedirectedEvent{
				EventId:     uuid2,
				ShortCode:   "xyz",
				ClickedAtMs: ts,
				VisitorIp:   "2001:db8::1",
				UserAgent:   "curl/8.0",
				Referer:     "",
			},
			wantIP:      net.ParseIP("2001:db8::1"),
			wantTime:    time.UnixMilli(ts).UTC(),
			wantReferer: "",
		},
		{
			name: "empty visitor_ip is rejected",
			input: &pb.UrlRedirectedEvent{
				EventId:     uuid3,
				VisitorIp:   "",
				ClickedAtMs: ts,
			},
			wantErr: true,
		},
		{
			name: "malformed visitor_ip is rejected",
			input: &pb.UrlRedirectedEvent{
				EventId:     uuid4,
				VisitorIp:   "not-an-ip",
				ClickedAtMs: ts,
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := protoToRedirectEvent(tt.input)

			if tt.wantErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)

			// The conversion generates a new UUIDv7 (uses input as random seed),
			// so we only verify it's a valid UUIDv7 with the correct version.
			require.Equal(t, uuid.Version(7), got.EventID.Version(), "EventID should be UUIDv7")
			require.Equal(t, tt.input.ShortCode, got.ShortCode)
			require.True(t, got.ClickedAt.Equal(tt.wantTime), "ClickedAt: got %v, want %v", got.ClickedAt, tt.wantTime)
			require.Equal(t, time.UTC, got.ClickedAt.Location(), "ClickedAt timezone should be UTC")
			require.True(t, got.VisitorIP.Equal(tt.wantIP), "VisitorIP: got %v, want %v", got.VisitorIP, tt.wantIP)
			require.Equal(t, tt.input.UserAgent, got.UserAgent)
			require.Equal(t, tt.wantReferer, got.Referer)
		})
	}
}
