#!/bin/sh
set -eu
# Local credentials are generated once, never checked into source or logged.
mkdir -p /run/cardlattice
for role in database-password contributor moderator metrics; do
  if [ ! -s "/run/cardlattice/$role" ]; then
    head -c 48 /dev/urandom | base64 | tr -d '\n/+=' > "/run/cardlattice/$role"
    printf '\n' >> "/run/cardlattice/$role"
  fi
done
printf 'postgres://cardlattice:%s@db:5432/cardlattice\n' "$(cat /run/cardlattice/database-password)" > /run/cardlattice/database-url
# Docker volume access is restricted to the local operator; service UIDs differ.
chmod 755 /run/cardlattice
chmod 644 /run/cardlattice/*
