#!/usr/bin/env sh

usage() {
	echo "flags:"
	echo "  -h --help"
	echo "  -c --container <container>"
	echo "  -u --user <user>"
	echo "  -p --password <password>"
	echo "  -d --database <database>"
	echo "  -h --host <host>"
	echo "  -f --file <mock_file>"
	exit "$1"
}

CONTAINER=""
USER="postgres"
PASSWORD="password"
DATABASE="strecklistan"
HOST="localhost"
FILE="$(dirname $0)/init.sql"

set -o errexit -o noclobber -o nounset

SHORTOPTS="c:u:p:h:d:f:"
LONGOPTS="help,container:,user:,password:,host:,database:,file:"

params="$(getopt -o "$SHORTOPTS" -l $LONGOPTS --name "$0" -- "$@")"
eval set -- "$params"

while true; do
	case "$1" in
		   --help)      usage 0        ; shift 1 ;;
		-c|--container) CONTAINER="$2" ; shift 2 ;;
		-u|--user)      USER="$2"      ; shift 2 ;;
		-p|--password)  PASSWORD="$2"  ; shift 2 ;;
		-d|--database)  DATABASE="$2"  ; shift 2 ;;
		-h|--host)      HOST="$2"      ; shift 2 ;;
		-f|--file)      FILE="$2"      ; shift 2 ;;
		--)             shift          ; break ;;
		 *) echo "Invalid argument: $1"; usage 1 ;;
	esac
done

if [ -n "$CONTAINER" ]; then
	set -x
	docker exec -i "$CONTAINER" \
		--env "PGPASSWORD=$PASSWORD" \
		psql -v ON_ERROR_STOP=1 -d "$DATABASE" -U "$USER" -h "$HOST" < "$FILE"
else
	export PGPASSWORD="$PASSWORD"
	set -x
	psql -v ON_ERROR_STOP=1 -d "$DATABASE" -U "$USER" -h "$HOST" < "$FILE"
fi

