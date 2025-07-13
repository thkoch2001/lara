#!/usr/bin/env sh

[ -z $PGDATA ] && echo "\$PGDATA not set" && exit 1

pg_ctl -l $PGDATA/logfile \
       -o "--unix_socket_directories=$PGDATA" \
       -o "--listen_addresses=''" \
       start
