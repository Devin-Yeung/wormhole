package tidb

import (
	"context"
	"database/sql"
	"testing"

	"github.com/Devin-Yeung/wormhole/analytics/sqlc"
	"github.com/pressly/goose/v3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/testcontainers/testcontainers-go/modules/tidb"

	_ "github.com/go-sql-driver/mysql"
)

func newTestTiDBContainer(ctx context.Context, tb testing.TB) (ctr *tidb.Container, shutdown func()) {
	ctr, err := tidb.Run(ctx,
		"pingcap/tidb:v8.4.0",
	)
	require.NoError(tb, err)

	shutdown = func() {
		err := ctr.Terminate(ctx)
		require.NoError(tb, err)
	}

	return ctr, shutdown
}

// NewTiDB creates a disposable TiDB instance and applies the analytics schema.
func NewTiDB(ctx context.Context, tb testing.TB) (db *sql.DB, shutdown func()) {
	tidbContainer, shutdownContainer := newTestTiDBContainer(ctx, tb)

	db, err := sql.Open("mysql", tidbContainer.MustConnectionString(ctx))
	require.NoError(tb, err)

	goose.SetBaseFS(sqlc.Migrations)

	err = goose.SetDialect("mysql")
	require.NoError(tb, err)

	err = goose.Up(db, "migrations")
	require.NoError(tb, err)

	shutdown = func() {
		err = db.Close()
		require.NoError(tb, err)
		shutdownContainer()
	}

	return db, shutdown
}

func TestMigration(t *testing.T) {
	ctx := context.Background()

	// Spin up TiDB and verify goose can apply and roll back the analytics schema.
	db, shutdown := NewTiDB(ctx, t)
	defer shutdown()

	// ping the database to ensure it's up and running
	err := db.Ping()
	assert.NoError(t, err)

	// also make sure the goose down migration works
	err = goose.Down(db, "migrations")
	assert.NoError(t, err)
}
