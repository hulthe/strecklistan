docker exec -u postgres -i "$1" psql -d laggit < init.sql
