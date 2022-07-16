# Universe 

## Overview

The Universe server is the central server to which all players, bots, and World servers connect and are managed by.

## Building

This software uses the typical Rust build system `cargo`. Get started with Rust at https://rustup.rs/.

Build a release version of the Universe server with `cargo build -r`, or run it directly from cargo with `cargo run`.

## Setting up the Universe

Upon running the Universe for the first time, a `universe.toml` file will be created in the present working directory. A few components must be set up before using the Universe server.

1) The IP address of the Universe server in `universe.toml` must be the same as the IP address that incoming clients will connect to.
2) The IP, port, and credentials for an active MySQL server need to be provided in `universe.toml`. Install, start, and configure a MySQL server if necessary.
   * The database (by default `aworld_universe`) needs to be created; the Universe server will not do it automatically.

The Universe will create a default account with the username `Administrator` and the password `welcome` automatically. You can log into this account with an AW 4 or AW 5 browser.

## Creating World licenses

Before a World will be able to join the Universe, a license for a world must be made. From within an AW browser, Select `Options` > `Universe` > `Worlds`. From the resulting window, you can configure a new World which you can then run using a World server.
