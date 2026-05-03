# Changelog

All notable changes to this project are documented in this file.

The format is based on Keep a Changelog, and the project follows Semantic Versioning.

## [0.7.0] - 2026-05-03

### Changed

- 升级 `orion-error` 至 0.8，适配 breaking API 变更。
- `PathReason` 改为 `#[derive(OrionError)]`，替代手写 `DomainReason` / `ErrorCode` impl。
  - `Brief` variant：`identity = "biz.path.brief"`，`code = 500`。
  - `Unified` variant：`#[orion_error(transparent)]` 委托给 `UnifiedReason`。
- 移除 `compat_traits::ErrorOweSource`：`.owe_sys_source()` / `.owe_res_source()` → `.source_err(reason, detail)` + `.conv_err()`。
- `UvsReason` → `UnifiedReason`，`UvsFrom` / `from_res()` → `PathReason::from(UnifiedReason::...)`。
- `DomainReason` / `ErrorCode` 导入路径从 crate root 移至 `orion_error::reason`。
- `ToStructError` 导入路径从 `traits_ext` 移至 `conversion`。
- `ConvErr` / `SourceErr` 从 `prelude` 导入，替代原有的 compat 路径。
- 移除 `thiserror` 在 `path.rs` 中的直接使用（由 `OrionError` derive 接管 Display / ErrorCode / ErrorIdentityProvider）。

### Dependencies

- `orion-error`: `0.7` → `0.8`

## [0.6.0] - 2026-04-24

### Changed

- 升级 `orion-error` 至 0.7，适配 `DomainReason` trait 与导入路径变更。
- `ErrorOwe` → `compat_traits::ErrorOweSource`，`ToStructError` → `traits_ext::ToStructError`。
- `owe_sys()`/`owe_res()` 替换为 `owe_sys_source()`/`owe_res_source()`。
- `.with()` → `.with_context()`，`.want()` → `.doing()`，消除 deprecated 警告。

## [0.5.0] - 2026-03-03

### Changed

- Package metadata refreshed (`name = "orion-infra"`, description/license/repository/keywords/categories).
- Config lifecycle now integrates with `orion_conf` and uses `OrionConfResult`.
- Dependencies upgraded/aligned:
  - `orion-error` -> `0.6`
  - added `orion_conf = 0.5` (`yaml`, `toml` features)
  - `thiserror = 2.0`, `log = 0.4`, `toml = 0.9`, `serde = 1.0`, `anyhow = 1.0`, `scopeguard = 1.2`, `flexi_logger = 0.31`

### Fixed

- `src/path.rs`: adapted to `orion-error 0.6` API (`UvsFrom` / `from_res` flow).
- `src/logging/configure.rs`: resolved `clippy::collapsible_if` under `-D warnings`.

### Docs

- Added and updated `README.md` and `CHANGELOG.md`.

## [0.2.0] - 2025-07-17

### Added

- Initial import of infrastructure utility modules (config/logging/path/types).
