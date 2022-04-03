#!/usr/bin/env bash

set -eou pipefail

function apply_cluster_state() {
    if [ "$CLUSTER_STATE" == "down" ]; then
        shutdown
    else
        shutdown && docker-compose -f $CLUSTER_SPEC_FILE up -d
    fi
}

function shutdown() {
    docker-compose -f $CLUSTER_SPEC_FILE down -v --remove-orphans
    docker container prune
}

CLUSTER_STATE="up"
CLUSTER_TYPE="single"
CLUSTER_SPEC_FILE="mongodb-single.yaml"

while [[ $# -gt 0 ]]; do
  key="$1"

  case $key in
    -s|--state)
      CLUSTER_STATE="$2"
      shift
      shift
      ;;
    -t|--type)
      CLUSTER_TYPE="$2"
      shift
      shift
      ;;
    *)
      shift
      ;;
  esac
done

case "$CLUSTER_TYPE" in
  single)
    CLUSTER_SPEC_FILE="mongodb-single.yaml"
    ;;

  replica-set)
    CLUSTER_SPEC_FILE="mongodb-replica-set.yaml"
    ;;
  *)
    echo "Unknown '$CLUSTER_TYPE' cluster type."
    ;;
esac

apply_cluster_state
