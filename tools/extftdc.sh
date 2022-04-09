#!/usr/bin/env bash

set -eou pipefail

OUTPUT_PATH=${1:-"./ftdc"}


echo "Extracting full-time diagnostic data capture files into $OUTPUT_PATH for each mongod instance..."

for container_id in $(docker ps --filter "name=mongo"  --format='{{.ID}}')
do
    docker cp "${container_id}:/data/db/diagnostic.data" "${OUTPUT_PATH}/${container_id}"
done

echo "Full-time diagnostic data capture files have been extracted successfully!"
