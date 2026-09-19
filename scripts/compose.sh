#!/bin/bash

set -euo pipefail

generate_compose_command() {
  # $1: cmd array ref
  # $2: db backend
  # $3: profile
  if [ $# -ne 3 ]; then
    echo "need at least 2 arguments: db backend, profile"
    return 1
  fi

  local -n _command=$1
  local _db=${2,,}
  local _profile=${3,,}
  _command=(docker compose -f docker-compose.yaml)

  if [ "$_db" == 'sqlite' ]; then
    _command+=(-f docker-compose.sqlite.yaml)
  else
    echo "invalid db backend: $_db"
    return 1
  fi

  if [ "$_profile" == 'dev' ]; then
    _command+=(-f docker-compose.dev.yaml)
  else
    echo "invalid profile: $_profile"
    return 1
  fi
}

main() {
  # $1: db backend
  # $2: profile

  if [ $# -lt 2 ]; then
    echo "usage: ./compose.sh sqlite dev"
    return 1
  fi

  generate_compose_command cmd $1 $2
  shift 2

  ${cmd[@]} $@
}

main $@
