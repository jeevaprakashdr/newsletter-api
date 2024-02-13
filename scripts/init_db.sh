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

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5433}"
DB_HOST="${POSTGRES_HOST:=127.0.0.1}" 

if [[ -z "${SKIP_DOCKER}" ]]; then
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
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