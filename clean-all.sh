#!/usr/bin/env sh

MEMBERS="acme-client common-utils job-execution persistence"

for DIR in $MEMBERS; do
  echo "Clearing dir: ${DIR}"
  if [ -d "${DIR}" ]; then
    cd "${DIR}" || exit 1
    pwd
    cargo clean
    cd - > /dev/null || exit
  else
    echo "Dir ${DIR} does not exist! skipping.."
  fi
done

exec cargo clean