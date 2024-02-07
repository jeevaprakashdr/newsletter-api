#!/usr/bin/env bash
set -x
set -eo pipefail

if ![ -x "$(command -v psql)"]; then
    echo "Error: psql is not installed"
    exit 1
fi

if ![ -x "$(command -v sqlx)"]; then
    echo "Error: sqlx-cli is not installed"
    exit 1
fi

IMAGE_NAME=newsletter-dev
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}" 

if [ $( docker ps -a | grep $IMAGE_NAME | wc -l ) -eq 0 ]; then
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        --name ${IMAGE_NAME} \
        -d postgres \
        postgres -N 1000
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "${DB_NAME}" -c "\q"; do
    echo "postgres database is still unavailable"
    sleep 1
done

echo "postgres database is available on port ${DB_PORT}"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run