package mysql

import (
	"context"
	"database/sql"
	"testing"

	assets "github.com/Devin-Yeung/wormhole/analytics"
	"github.com/pressly/goose/v3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/testcontainers/testcontainers-go/modules/mysql"

	_ "github.com/go-sql-driver/mysql"
)

func newTestMySQLContainer(ctx context.Context, t *testing.T) (container *mysql.MySQLContainer, shutdown func()) {
	container, err := mysql.Run(ctx,
		"mysql:8.4",
		mysql.WithDatabase("wormhole"),
		mysql.WithUsername("root"),
		mysql.WithPassword("root"),
	)
	require.NoError(t, err)

	shutdown = func() {
		err := container.Terminate(ctx)
		require.NoError(t, err)
	}

	return container, shutdown
}

// create a new mysql container with migration applies
func NewMysql(ctx context.Context, t *testing.T) (db *sql.DB, shutdown func()) {
	mysqlContainer, shutdownContainer := newTestMySQLContainer(ctx, t)

	dsn := mysqlContainer.MustConnectionString(ctx)

	db, err := sql.Open("mysql", dsn)
	require.NoError(t, err)

	goose.SetBaseFS(assets.Migrations)

	err = goose.SetDialect("mysql")
	require.NoError(t, err)

	err = goose.Up(db, "sqlc/migrations")
	require.NoError(t, err)

	shutdown = func() {
		err = db.Close()
		require.NoError(t, err)
		shutdownContainer()
	}

	return db, shutdown
}

func TestMigration(t *testing.T) {
	ctx := context.Background()

	// spin up a mysql container and apply migration
	db, shutdown := NewMysql(ctx, t)
	defer shutdown()

	// ping the database to ensure it's up and running
	err := db.Ping()
	assert.NoError(t, err)
}
