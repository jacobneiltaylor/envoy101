setup: (build "1")
    LESSON=1 docker compose pull

build IDX:
    LESSON={{ IDX }} docker compose build

start IDX: (build IDX)
    LESSON={{ IDX }} docker compose up

stop:
    docker compose down
