# orion-infra

[![CI](https://github.com/galaxio-labs/orion-infra/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/galaxio-labs/orion-infra/actions)
[![Coverage Status](https://coveralls.io/repos/github/galaxio-labs/orion-infra/badge.svg?branch=main)](https://coveralls.io/github/galaxio-labs/orion-infra?branch=main)
[![crates.io](https://img.shields.io/crates/v/orion-infra.svg)](https://crates.io/crates/orion-infra)
[![Downloads](https://img.shields.io/crates/d/orion-infra.svg)](https://crates.io/crates/orion-infra)
[![docs.rs](https://docs.rs/orion-infra/badge.svg)](https://docs.rs/orion-infra)
[![Dependencies](https://deps.rs/repo/github/galaxio-labs/orion-infra/status.svg)](https://deps.rs/repo/github/galaxio-labs/orion-infra)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 2024 edition](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org)
[![GitHub stars](https://img.shields.io/github/stars/galaxio-labs/orion-infra.svg)](https://github.com/galaxio-labs/orion-infra/stargazers)

`orion-infra` is a Rust infrastructure helper crate.

It provides:

- config file lifecycle helpers (load/init/save/try_load)
- logging setup based on `flexi_logger`
- path utility helpers with typed errors
- a small auto-exit logging guard macro

## Install

```toml
[dependencies]
orion-infra = "0.5"
```

> Crate import path is `orion_infra` in Rust code.

## Modules

- `orion_infra::config`
  - `ConfigLifecycle`
  - `backup_clean`, `clear_file`, `read_file`, `save_data`
- `orion_infra::logging`
  - `LogConf`, `Output`, `configure_logging`, `BoolFlag`
  - macro: `auto_exit_log!`
- `orion_infra::path`
  - `make_clean_path`, `ensure_path`, `make_new_path`, `get_sub_dirs`
  - `PathReason`, `PathResult`
- `orion_infra::types`
  - `AnyResult<T>`

## Quick Start

### Logging

```rust
use orion_infra::logging::{configure_logging, LogConf};

fn init_log() {
    let conf = LogConf::new_console("info").rec_positon();
    configure_logging(&conf).expect("init logger");
}
```

### Config lifecycle (TOML)

```rust
use orion_infra::config::ConfigLifecycle;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConf {
    app_name: String,
    workers: u16,
}

fn demo_conf() {
    let path = "output/app.toml";
    let conf = AppConf {
        app_name: "demo".into(),
        workers: 4,
    };

    conf.init(path).expect("init config");
    let loaded = AppConf::load(path).expect("load config");
    let _try_loaded = AppConf::try_load(path);

    println!("{}", loaded.app_name);
}
```

### Path helpers

```rust
use orion_infra::path::{ensure_path, get_sub_dirs, make_clean_path, make_new_path};
use std::path::Path;

fn demo_path() {
    make_clean_path(Path::new("output/demo")).expect("clean path");
    ensure_path("output/demo/sub").expect("ensure path");
    let _ = make_new_path(Path::new("output/new_dir"));
    let _ = get_sub_dirs(Path::new("output"));
}
```

### Auto-exit log macro

```rust
use orion_infra::auto_exit_log;

fn run_task() {
    let mut flag = auto_exit_log!(
        log::info!("task success"),
        log::error!("task failed")
    );

    // do work...
    flag.mark_suc();
}
```

## Development

```bash
cargo check
cargo clippy -- -D warnings
cargo test
```

## License

MIT
