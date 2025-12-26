podman run \
  --name questdb \
  --replace \
	-p 9000:9000 \
	-v ./db_volume:/var/lib/questdb \
	docker.io/questdb/questdb:latest
