#!/usr/bin/env bash

START_WORKING_DIR=$(pwd)
docker container run -d -e MYSQL_ROOT_PASSWORD=dev --name dax-mysql --rm -p 3307:3306 -p 33070:33060 mysql:9
echo "Waiting for MySQL server to start..."
sleep 25

for migration_dir in $(find . -name "migrations");
do
  cd "${migration_dir}"/.. || exit 1
  database_name=$(echo "${migration_dir}" | sed -E "s/.\/dys-svc-(.*)\/migrations/\1/g")
  sqlx database setup --database-url="mysql://root:dev@localhost:3307/dax_${database_name}"
done

cd "${START_WORKING_DIR}" || exit 1