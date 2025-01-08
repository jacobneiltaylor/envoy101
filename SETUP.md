# Lab Setup

This is a quick lab setup guide for our Envoy 101 session!
You can perform these steps while listening along, or in the first lab which is dedicated to getting this working for everyone.

## Prerequisites

 - A Docker or other Compose-compatible environment set up on your machine
   - [Docker Desktop](https://www.docker.com/products/docker-desktop/) is the recommended option
   - [Colima](https://github.com/abiosoft/colima) is another option
 - Basic understanding of Python

## Installing Just (optional)

Just is a command runner I use to make the commands needed to run the lab a lot simpler.
It is also completely optional if you don't want to install random binaries on your machine.
It's available through a few package managers:

 - Homebrew: `brew install just`
 - Fedora: `dnf install just`
 - Ubuntu 24.04: `apt install just`
 - Arch: `pacman -S just`
 - openSUSE: `zypper in just`

Alternatively, compile from scratch:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install just
```

## Precaching the required images

To make the following labs faster to start, you can run the following commands to build and precache the necessary container images:

```
just setup
```

For those without Just:

```
LESSON=1 docker compose pull
LESSON=1 docker compose build
```

## Running the n-th lesson

To run the n-th lesson, simply run the following command:

```
# to start the lab in the background
just start n
# to stop the lab
just stop
```

Or without just:

```
# to start the lab in the background
LESSON=1 docker compose build
LESSON=1 docker compose up
# to stop the lab
docker compose down
```
