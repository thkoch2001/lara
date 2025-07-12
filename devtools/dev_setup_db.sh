#!/usr/bin/env bash

initdb
pg_ctl -l $PGDATA/logfile -o "--unix_socket_directories=$PGDATA" start
createdb lara
diesel migration --migration-dir crates/main/migrations run
