#!/bin/bash

set -euxo pipefail

DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

source "$DIR/lib.sh"

get_migration_command() {
  # $1: cmd array ref
  # $2: service name
  #
  if [ $# -ne 2 ]; then
    echo "need at least 2 arguments: cmd array, service name"
    return 1
  fi

  local -n _command=$1
  local _service=${2,,}
  if [ $_service == 'auth' ]; then
    _command=("/app/target/debug/langcities-auth-migration")
  elif [ $_service == 'dc' ]; then
    _command=("/app/target/debug/langcities-dc-migration")
  else
    echo "unknown service: $2"
    return 1
  fi
}

main() {
  # $1: auth/dc
  # $2...: further flags
  # ensure_sea_orm_cli
  if [ $# -lt 1 ]; then
    echo "Usage: ./sea-migrate.sh <auth/dc>"
    return 1
  fi
  local _db_url="$(get_db_url $1)"
  echo "db_url: $_db_url"
  get_migration_command cmd $1
  shift 1
  ${cmd[@]} -u "$_db_url" $@
}

main $@
