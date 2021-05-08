#!/bin/sh

if [ $# -ne 3 ]
then
	echo "Invalid arguments supplied!"
	echo ""
	echo "Usage:"
	echo "  ./run.sh <container name> <token> <database directory>"
	exit 1
fi

CONTAINER_NAME="$1"
TOKEN="$2"
DB_DIR="$3"

docker build . -t narc
docker container stop "$CONTAINER_NAME"
docker container rm "$CONTAINER_NAME"
docker run \
	--detach \
	--name "$CONTAINER_NAME" \
	-e "DATABASE_URL=sqlite:/db/narc.db" \
	-e "DISCORD_TOKEN=$TOKEN" \
	-e "RUST_LOG=narc=warn" \
	-v "$DB_DIR":/db \
	narc
