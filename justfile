dev-up:
  docker compose \
    -f ./infra/dev/docker-compose.yml \
    up \
    -d --force-recreate --remove-orphans

logs:
  lnav \
    docker://wormhole-redis-master \
    docker://wormhole-redis-slave \
    docker://wormhole-redis-sentinel \
    docker://wormhole-mysql

compile-go-proto:
    protoc proto/**/*.proto \
        --go_out=./analytics/pb \
        --go-grpc_out=./analytics/pb \
        --go_opt=paths=source_relative \
        --go-grpc_opt=paths=source_relative \
        --proto_path=proto # define the proto path for imports
