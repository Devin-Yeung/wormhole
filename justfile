up:
  docker compose \
    -f ./infra/dev/docker-compose.yml \
    up \
    -d --force-recreate --remove-orphans

down:
  docker compose \
    -f ./infra/dev/docker-compose.yml \
    down

logs:
  docker compose \
    -f ./infra/dev/docker-compose.yml \
    logs -f | lnav

gateway:
  watchexec \
    -e rs \
    -- cargo run --quiet \
    --bin gateway \
    | lnav

tinyflake:
  watchexec \
    -e rs \
    -- cargo run --quiet \
    --bin tinyflake \
    | lnav

compile-go-proto:
    update-go-pb
