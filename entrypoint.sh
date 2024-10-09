#!/bin/sh
set -e

if [ -z "$ECS_CONTAINER_METADATA_URI_V4" ]; then
  echo "ECS_CONTAINER_METADATA_URI_V4 is not set."
  exit 1
fi

METADATA=$(curl -s ${ECS_CONTAINER_METADATA_URI_V4}/task)

CONTAINER_ID=$(echo $METADATA | jq -r '.Containers[0].DockerId')

export WORKER_NAME="worker-${CONTAINER_ID}"

exec "$@"

