docker build . -t narc
docker container stop narc
docker container rm narc
docker run \
	--detach \
	--name narc \
	-e "DATABASE_URL=sqlite:narc.db" \
	-e "DISCORD_TOKEN=$TOKEN" \
	-e "RUST_LOG=narc=warn" \
	-v "$PWD/narc.db":/app/narc.db \
	narc
