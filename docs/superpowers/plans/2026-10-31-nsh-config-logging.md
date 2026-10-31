# NSH 配置与日志对齐实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` to implement this plan task by task. Steps use checkbox syntax for tracking.

**Goal:** 让 Rust 与 TSX NSH UI 共用 Slint 兼容的配置文件和保存行为，并恢复原版控制台/文件日志。

**Architecture:** `src/config.rs` 持有 Slint 兼容的 TOML schema、默认值和读写；Rust native module 暴露给两端的同步 load/save API。旧 NSH 配置键忽略并清除，不迁移。两个根组件在配置变化后防抖保存，并在 cleanup 时保存完整状态快照。`src/logger.rs` 在 native module 注册时初始化一次，记录到控制台和原版日志文件。

**Tech Stack:** Rust、serde/TOML、NSH native modules/Storage/cleanup、log/env_logger/chrono、NSH headless UI tests。

---

### Task 1: Config schema、默认值与 TOML 读写

**Files:**
- Create: `src/config.rs`
- Modify: `Cargo.toml`, `Cargo.lock`
- Test: `src/config.rs` unit tests

- [x] **Step 1: 写失败测试，固定 Slint 默认值和 TOML 名称**

```rust
#[test]
fn defaults_match_slint() {
    let config = Config::default();
    assert_eq!(config.song_dir, music_home().join("Music"));
    assert_eq!(config.play_mode, PlayMode::InOrder);
    assert_eq!(config.sort_key, SortKey::BySongName);
    assert!(config.sort_ascending);
    assert_eq!(config.volume, 1.0);
    assert!(config.follow_system_theme);
    assert!(config.show_spectrum);
    assert_eq!(config.current_page, 0);
    assert!(!config.sidebar_expanded);
    assert_eq!(config.gallery_viewport_y, 0.0);
}

#[test]
fn toml_round_trip_keeps_slint_enum_names() {
    let config = Config::default();
    let text = toml::to_string_pretty(&config).unwrap();
    assert!(text.contains("play_mode = \"InOrder\""));
    assert!(text.contains("sort_key = \"BySongName\""));
    assert_eq!(toml::from_str::<Config>(&text).unwrap(), config);
}
```

- [x] **Step 2: 运行测试确认 RED**

Run: `rtk cargo test --locked config::tests`

Expected: compile/test failure，因为 Config 和 Slint 对应枚举尚不存在。

- [x] **Step 3: 实现 Config 模型和可注入路径的读写函数**

定义全部 Slint 字段：`song_dir`、`current_song_path`、`progress`、`play_mode`、`sort_key`、`sort_ascending`、`lang`、`light_ui`、`follow_system_theme`、`volume`、`show_spectrum`、`current_page`、`sidebar_expanded`、`gallery_viewport_y`。枚举使用 Slint 名称 `InOrder/Recursive/Random` 和 `BySongName/BySinger/ByDuration`。实现 `Config::default()`、`load_at(path)`、`save_at(path)` 和默认路径 `home/.config/zeedle/config.toml`。若设置测试环境变量 `ZEEDLE_CONFIG_PATH`，仅在该变量存在时使用它隔离测试；默认路径不变。复用原版依赖版本：`home 0.5.11`、`toml 0.9.5`，并保持锁文件可用。

- [x] **Step 4: 补齐缺失文件、损坏 TOML 和旧字段测试，再运行**

Run: `rtk cargo test --locked config::tests`

Expected: 默认值、TOML round-trip、缺文件回默认值、旧 Slint 配置缺少带 serde 默认值字段时兼容。

### Task 2: Config native API 和旧键重置

**Files:**
- Modify: `src/config.rs`, `src/lib.rs`
- Test: `src/config.rs` load tests and NSH headless tests

- [x] **Step 1: 写失败测试，验证 Slint TOML 优先并清除旧 Storage**

在 headless NSH 测试中给 `open` 注入与已存在非默认 Slint TOML 冲突的旧 `zeedle.*` Storage 键；断言 TOML 值生效且旧键被清除。

- [x] **Step 2: 运行测试确认 RED**

Run: `rtk python tests/test_config_persistence.py`

Expected: 当前 Rust/TSX UI 仍读取旧 Storage 值，测试失败。

- [x] **Step 3: 实现纯 TOML load/save 与 AppConfig native API**

`Config::load()` 只读取 Slint TOML；文件缺失时返回 Slint 默认值。已知旧 NSH 配置键在启动时清除，不解析或迁移。暴露同步 `AppConfig.load()`/`AppConfig.save(json)`，测试用 `ZEEDLE_CONFIG_PATH` 指向临时 TOML；默认路径仍是 Slint 路径。

- [x] **Step 4: 运行 Config 和旧键重置测试确认 GREEN**

Run: `rtk cargo test --locked config::tests`

Run: `rtk python tests/test_config_persistence.py`

Expected: TOML/default 行为正确，旧键不影响配置，native API 可读取和保存完整快照。

### Task 3: Logger 和 backend 日志

**Files:**
- Create: `src/logger.rs`
- Modify: `Cargo.toml`, `Cargo.lock`, `src/lib.rs`
- Test: `src/logger.rs` unit tests

- [x] **Step 1: 写失败测试，覆盖格式、双写和轮转**

测试日志行包含本地时间、level、源文件、行号；测试 `MultiWriter` 将同一缓冲写到两个 sink；用 `File::set_len(10 * 1024 * 1024 + 1)` 验证超限文件会在初始化时清理，未超限文件保留。

- [x] **Step 2: 运行 logger 测试确认 RED**

Run: `rtk cargo test --locked logger::tests`

Expected: logger 实现缺失导致失败。

- [x] **Step 3: 实现 Slint 同款 logger**

增加 `chrono 0.4.42`、`env_logger 0.11.8`、`log 0.4.28`（遵从已锁定的兼容版本）。日志 append 到 `~/.zeedle.log` 并写 stdout；默认 `Info`；超过 10 MiB 清理；格式 `[timestamp | level | file:line] --> message`。panic hook 先记录 panic，再调用先前 hook。使用 `OnceLock` 保证 native module 只初始化一次，并在 host 启动 UI 前注册。

- [x] **Step 4: 在 backend 关键路径加入日志**

记录配置 load/default creation/save、曲库扫描结果、音频设备不可用、曲目 load/play、暂停、seek 失败和音量变化；不在 120 ms 状态轮询中逐条输出。

- [x] **Step 5: 运行 logger 测试确认 GREEN**

Run: `rtk cargo test --locked logger::tests`

Expected: 实际 logger 的格式与双写、文件轮转测试通过；`OnceLock` 保证 logger 初始化幂等。

### Task 4: Rust UI 使用完整 Config

**Files:**
- Modify: `src/ui/mod.rs`, `src/lib.rs`
- Test: `src/ui/mod.rs` config mapping tests, `tests/rust-platform-render.test.tsx`

- [x] **Step 1: 写 Rust UI 与 Config 枚举映射测试**

验证 `InOrder/Recursive/Random` 与 UI 字符串 `order/repeat/random`，以及 `BySongName/BySinger/ByDuration` 与 `song_name/singer/duration_secs` 的双向转换。

- [x] **Step 2: 加载 Config 后再初始化 signals**

按 Config 初始化目录、音量、模式、排序、语言/主题、频谱、页面和侧栏。曲库扫描后恢复有效的当前曲目及其进度；失效路径回退到首曲并从 0 开始；空曲库保留偏好设置但清空歌曲运行态。将 audio backend 的默认音量从 `0.8` 改成 Slint 的 `1.0`。

- [x] **Step 3: 写并接入 cleanup 保存**

在 Rust 根组件中注册 `nsh_core::element::cleanup`，清理时从 signals 和 `Music::state()` 创建完整 Config 并保存；保存错误经 logger 记录，不 panic。

- [x] **Step 4: 实现曲库滚动位置的记录和恢复**

给 Gallery virtual list 添加专用 `ElementRef`/`on_scroll`，用 `ScrollEvent.offset.y` 更新 `gallery_viewport_y`；曲库模型装载后再调用 `scroll_to`。Search 列表不得更新 Gallery 的偏移。

- [x] **Step 5: 确认 Rust 暗色/浅色渲染**

Run: `rtk python scripts/with-rust-root.py nsh test tests/rust-platform-render.test.tsx --target windows`

Expected: Rust 多页渲染通过；检查暗色和浅色曲库、设置页截图的文字、图标、输入控件可读。

### Task 5: TSX UI 共用 Config

**Files:**
- Modify: `App.tsx`
- Create: `tests/config-persistence.test.tsx`, `tests/rust-config-persistence.test.tsx`, `tests/test_config_persistence.py`

- [x] **Step 1: 写 TSX/Rust 根组件的旧键重置与完整状态 round-trip 测试**

每个测试用非默认 Slint TOML 和冲突的 `open(..., { storage: { ...legacyKeys } })` 启动，确认 TOML 优先、旧键已清除；交互切换侧栏后等待防抖保存。Python 包装测试为每个 `nsh test` 进程设置独立的 `ZEEDLE_CONFIG_PATH` 临时文件，并用 Python `tomllib` 检查更新后的完整快照，避免测试读写用户配置。

- [x] **Step 2: 运行测试确认 RED**

Run: `rtk nsh test tests/config-persistence.test.tsx --target windows`

Run: `rtk python scripts/with-rust-root.py nsh test tests/rust-config-persistence.test.tsx --target windows`

Expected: TSX/Rust 尚未调用统一 Config API，且仍读取旧 Storage，断言失败。

- [x] **Step 3: 改造 TSX 启动加载和 cleanup 保存**

在创建 UI signals 前读取 Config；移除 `App.tsx` 中对配置字段的独立 `Storage.getItem/setItem`，只保留清理旧键的 `removeItem`；cleanup 时保存当前音乐目录/曲目/进度、播放和排序模式、语言/主题、音量/频谱、页面/侧栏及 Gallery offset。

- [x] **Step 4: 给 TSX Gallery List 增加 offset 保存与恢复**

使用 List 的 scroll event 记录偏移；先加载数据，再恢复位置。不得把 Search List 的 offset 写入 Gallery 配置。

- [x] **Step 5: 运行两端旧键重置和渲染测试确认 GREEN**

Run: `rtk nsh test tests/config-persistence.test.tsx --target windows`

Run: `rtk python scripts/with-rust-root.py nsh test tests/rust-config-persistence.test.tsx --target windows`

Run: `rtk nsh test tests/platform-render.test.tsx --target windows`

Expected: 两端忽略旧键并清除它们，所选歌曲、进度和交互后的侧栏状态写回隔离 TOML；浅色和深色截图均通过。

### Task 6: 完整验证

**Files:**
- Review: `Cargo.toml`, `Cargo.lock`, `src/config.rs`, `src/logger.rs`, `src/lib.rs`, `src/ui/mod.rs`, `App.tsx`, the new tests

- [x] **Step 1: 运行格式、单元和 NSH 检查**

Run: `rtk cargo fmt --check`

Run: `rtk cargo test --locked`

Run: `rtk nsh check -v`

Run: `rtk python tests/test_config_persistence.py`

Expected: 所有命令退出码为 0；Slint TOML、日志轮转、旧键重置、两端持久化和浅/深色 UI 测试均执行。

- [x] **Step 2: Review final diff and whitespace**

Run: `rtk git diff --check; rtk git status --short; rtk git diff --stat`.

Expected: 没有格式错误或与配置/日志无关的改动。
