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

