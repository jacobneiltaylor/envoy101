FROM rust:1.75.0

RUN mkdir /mnt/ipc

WORKDIR /usr/src/app

RUN mkdir ./src

COPY ./src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .

RUN cargo install --path .

CMD ["extauthz"]
