#!/bin/bash

set -euo pipefail

main() {
  # $1: db backend
  # $2: profile
  # $3: service

  if [ $# -lt 2 ]; then
    echo "usage: ./db-codegen.sh sqlite dev auth/dc"
    return 1
  fi

  $(dirname $0)/compose.sh $1 $2 run --rm dev-build \
    /app/scripts/internal/sea-codegen.sh $3
}

main $@
