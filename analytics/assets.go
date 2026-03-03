package assets

import (
	"embed"
)

//go:embed sqlc/migrations
var Migrations embed.FS
