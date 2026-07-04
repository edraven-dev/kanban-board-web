#!/usr/bin/env bash
# Runs the given command, then removes this project's testcontainers containers.
#
# Under Docker, testcontainers' Ryuk reaper already removes them; under Podman Ryuk
# is unreliable, so we prune by our own label to cover both runtimes. The label is
# project-scoped so we never touch containers from other projects.
set -uo pipefail

"$@"
status=$?

cli=$(command -v docker || command -v podman || true)
if [ -n "$cli" ]; then
  ids=$("$cli" ps -aq --filter "label=com.kanban.test=1" 2>/dev/null || true)
  if [ -n "$ids" ]; then
    "$cli" rm -f $ids >/dev/null 2>&1 || true
  fi
fi

exit "$status"
