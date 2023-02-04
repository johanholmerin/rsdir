#!/usr/bin/env sh

# For testing the error handling when the editor is killed by a signal, in this
# case by the script killing itself. See the `editor_killed` test

kill $$
