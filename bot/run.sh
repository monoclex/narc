#!/bin/sh

if [ $# -ne 4 ]
then
        echo "Invalid arguments supplied!"
        echo ""
        echo "Usage:"
        echo "  ./run.sh <container name> <token> <database directory> <image name>"
        exit 1
fi

CONTAINER_NAME="$1"
TOKEN="$2" 
DB_DIR="$3"
IMAGE_NAME="$4"

docker build . -t "$IMAGE_NAME"
docker container stop "$CONTAINER_NAME"
docker container rm "$CONTAINER_NAME"
docker run \
        --detach \
        --name "$CONTAINER_NAME" \
        -e "DATABASE_URL=sqlite:/db/narc.db" \
        -e "DISCORD_TOKEN=$TOKEN" \
        -e "RUST_LOG=narc=warn" \
        -v "$DB_DIR":/db \
        "$IMAGE_NAME"
