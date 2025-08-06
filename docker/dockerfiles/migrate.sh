#!/bin/bash

MIGRATION_DIR=$1

if [[ -z "$MYSQL_USER" ]]; then
  echo "MYSQL_USER not set! Aborting"
  exit 1
fi

if [[ -z "$MYSQL_PASS" ]]; then
  echo "MYSQL_PASS not set! Aborting"
  exit 1
fi

if [[ -z "$MYSQL_HOST" ]]; then
  echo "MYSQL_HOST not set! Aborting"
  exit 1
fi

for migration_dir in $(find ${MIGRATION_DIR} -type d -not -name migrations);
do
  database_name=$(echo ${migration_dir} | sed -E "s/\/migrations\/(.*)/\1/g")
  echo "Creating database ${database_name}..."
  sqlx database create \
    --database-url=mysql://${MYSQL_USER}:${MYSQL_PASS}@${MYSQL_HOST}/dax_${database_name} \
  || exit 1

  echo "Running migrations..."
  sqlx migrate run \
    --database-url=mysql://${MYSQL_USER}:${MYSQL_PASS}@${MYSQL_HOST}/dax_${database_name} \
    --source ${migration_dir} \
  || exit 1
done
exit 0