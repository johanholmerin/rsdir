#!/usr/bin/env sh

# Runs `ed` using the script from the `ED_SCRIPT` environment variable
# stdout & stderr are both supresssed so as to not pollute the output from rsdir

echo "$ED_SCRIPT" | ed "$@" >/dev/null 2>/dev/null
