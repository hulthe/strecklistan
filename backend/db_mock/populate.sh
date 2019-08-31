#!/bin/sh
export DOCKER_NAME="$1"

if [ -z "$DOCKER_NAME" ]; then
	echo "Please specify the name/id of the database container."
	exit 1
fi

docker exec -u postgres -i "$1" psql -d laggit < init.sql
