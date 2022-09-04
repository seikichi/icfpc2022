#!/bin/sh
exec cargo flamegraph --bin cli -- "$@"
