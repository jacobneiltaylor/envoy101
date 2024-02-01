build IDX:
    LESSON={{ IDX }} docker compose build

start IDX: (build IDX)
    LESSON={{ IDX }} docker compose up

stop:
    docker compose down
