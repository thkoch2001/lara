#!/usr/bin/env sh

initdb
dev_start_db.sh
createdb lara
diesel migration --migration-dir crates/main/migrations run
