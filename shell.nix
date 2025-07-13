# copied from https://nixos.wiki/wiki/Rust
# https://discourse.nixos.org/t/set-up-a-simple-postgres-instance/60280

{ pkgs ? import <nixpkgs> { }, datadir ? "/dev/shm/dev_datadir_lara" }:
let
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
  libPath = with pkgs; lib.makeLibraryPath [
    # load external libraries that you need in your rust project here
  ];
in
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    #clang
    # Replace llvmPackages with llvmPackages_X, where X is the latest LLVM version (at the time of writing, 16)
    #llvmPackages.bintools
    (diesel-cli.override {
      sqliteSupport = false;
      mysqlSupport = false;
    })
    rustup
    libpq
    openssl
    pkg-config
    postgresql
  ];
  RUSTC_VERSION = overrides.toolchain.channel;
  # https://github.com/rust-lang/rust-bindgen#environment-variables
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
    export PATH=$PATH:$(pwd)/devtools

    export DATADIR=${datadir}
    mkdir -p $DATADIR

    export PGDATA=$DATADIR/postgres
    export RUST_BACKTRACE=1
    export RUST_LOG=lara

    # vars used by this project's code
    export FROM=larabot@koch.ro
    export BOT_NAME=larabot
    export ARCHIVE_DIR=$DATADIR/archive
    mkdir -p $ARCHIVE_DIR

    # used by postgres client cli
    export PGHOST=$PGDATA
    export PGDATABASE=lara

    # used by lara and postgres initdb, pg_ctl
    export DATABASE_URL=postgresql:///''${PGDATABASE}?application_name=lara\&host=$PGDATA

    alias dev_pg_start='pg_ctl -l $PGDATA/logfile -o "--unix_socket_directories=$PGDATA" start'
    alias dev_pg_status='pg_ctl status'
    alias dev_pg_stop='pg_ctl stop'
  '';
  # Add precompiled library to rustc search path
  RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
    # add libraries here (e.g. pkgs.libvmi)
  ]);
  LD_LIBRARY_PATH = libPath;
  # Add glibc, clang, glib, and other headers to bindgen search path
  BINDGEN_EXTRA_CLANG_ARGS =
    # Includes normal include path
    (builtins.map (a: ''-I"${a}/include"'') [
      # add dev libraries here (e.g. pkgs.libvmi.dev)
      # pkgs.glibc.dev
    ])
    # Includes with special directory paths
    ++ [
      #''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
      #''-I"${pkgs.glib.dev}/include/glib-2.0"''
      #''-I${pkgs.glib.out}/lib/glib-2.0/include/''
    ];
}
