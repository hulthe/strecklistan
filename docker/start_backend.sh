#!/usr/bin/env sh

printf "Waiting for database \"db:5432\"..."
while ! nc -z db 5432; do
	sleep 1
	printf "."
done
echo

set -x

diesel setup

db_mock/populate.sh \
	--host db \
	--user postgres \
	--password password \
	--database strecklistan \
	--file db_mock/init.sql

cargo watch -x run
