# NSH 配置与日志对齐设计

## 目标

让 NSH Rust UI 和 TSX UI 共用一套配置保存/加载系统，并使 Rust 后端的文件日志行为与原版 Slint UI 一致。

## 原版 Slint 行为

- 配置路径为 `~/.config/zeedle/config.toml`。
- 配置包含音乐目录、当前歌曲路径、播放进度、播放模式、排序字段和方向、语言、浅色/深色偏好、跟随系统主题、音量、频谱开关、当前页面、侧栏展开状态和曲库滚动位置。
- 缺少配置文件或 TOML 无法解析时使用 `Config::default()`。默认值包括 `~/Music`、顺序播放、按歌名升序、系统主题、音量 `1.0`、频谱开启、曲库页面、侧栏折叠和滚动位置 `0`。
- 启动时恢复设置；歌曲路径有效时恢复歌曲及进度，否则选择曲库首曲。空曲库保留用户设置并清空歌曲运行状态。
- 关闭窗口时保存完整配置快照。
- 日志写入 `~/.zeedle.log` 和控制台，默认 `Info` 级别，格式包含本地时间、级别、源文件和行号。启动时若日志超过 10 MiB，则清空旧文件。

## NSH 当前差异

Rust 和 TSX UI 在同一个 NSH `Storage` 中保存少量键值，包括音乐目录、音量、播放模式、频谱、语言和主题。当前没有 Slint TOML 文件兼容性，也未保存当前歌曲/进度、排序、页面、侧栏状态和曲库滚动位置；UI 与后端的默认音量为 `0.8`。Rust 后端没有应用日志器。

## 方案选择

1. **共用 Slint 兼容 TOML（选定）**：Rust 后端提供配置 load/save API，Rust 和 TSX UI 都使用同一文件；缺少 TOML 时将旧 NSH Storage 键迁移进来。
2. **只扩展 NSH Storage**：实现更多设置保存，但不兼容原版 Slint 文件路径和格式。
3. **Rust 与 TSX 分别管理配置**：改动局部，但继续造成两个 NSH UI 的状态分叉。

选择方案 1，保留 Slint 配置文件的字段名、枚举值和路径，使原有配置可以被 NSH 读取，之后也可由 Slint 读取。NSH Storage 是一次性迁移来源；迁移成功后，TOML 是两个 UI 唯一的配置写入源。

## 配置架构

### 配置模型

在 `src/config.rs` 定义 serde `Config`，字段和 Slint 版本一致：

```text
song_dir: PathBuf
current_song_path: Option<PathBuf>
progress: f32
play_mode: InOrder | Recursive | Random
sort_key: BySongName | BySinger | ByDuration
sort_ascending: bool
lang: String
light_ui: bool
follow_system_theme: bool
volume: f32
show_spectrum: bool
current_page: i32
sidebar_expanded: bool
gallery_viewport_y: f32
```

默认值保持 Slint 现状；特别是音乐目录 `~/Music`、顺序播放、按歌名升序、音量 `1.0`、`lang` 为空字符串、主题跟随系统、频谱开启、曲库页面、侧栏折叠和曲库滚动位置 `0`。空 `lang` 表示 Slint 的默认翻译；NSH 在当前支持的语言集合中使用英文回退。

### 加载和迁移

1. 初始化时读取 `~/.config/zeedle/config.toml`。
2. TOML 存在且有效时，它是唯一配置来源；旧 Storage 值不覆盖它。
3. TOML 不存在时，先读取 NSH Rust/TSX 共用的 Storage 键并映射到 Config，再用 Slint 默认值补齐没有旧键的字段。映射包括：
   - `zeedle.music-directory` → `song_dir`
   - `zeedle.volume` → `volume`
   - `zeedle.play-mode`: `order`/`repeat`/`random` → `InOrder`/`Recursive`/`Random`
   - `zeedle.show-spectrum` → `show_spectrum`
   - `zeedle.language` → `lang`
   - `zeedle.light-theme` → 手动主题；缺失时 `follow_system_theme=true`
4. 成功写入 TOML 后删除已迁移的旧键。写入失败时保留旧键并记录错误。
5. TOML 解析失败时记录警告并使用默认值；读取时不立即覆盖坏文件，正常关闭时按 Slint 行为保存当次完整快照。文件读取或目录/文件写入失败时记录错误，不因配置故障阻止 UI 启动。

### Rust 和 TSX 接入

在现有 Rust native module 中暴露同步配置读取和保存方法。两个 UI 在初始化时都读取同一份 Config，并据此设定设置、播放、排序和页面状态。曲库加载后恢复有效的 `current_song_path` 和 `progress`；无效路径时使用首曲并从零开始；空曲库时不清除已保存的用户偏好。

两端都在根组件 cleanup 中保存完整快照。保存包括播放后端当前进度和音量、选中歌曲、全部设置及 Gallery 滚动位置；曲库滚动位置在歌曲模型装载后恢复，避免空模型布局重置位置。

NSH Storage 仅用于首次迁移，不再作为之后的配置写入源。

## 日志架构

在 `src/logger.rs` 实现应用 logger，并在 app native module 注册时初始化一次：

- 日志路径：`~/.zeedle.log`；自动创建父目录，追加写入。
- 文件超过 10 MiB 时，在启动时清空后重新写入。
- 同时写控制台和文件，默认级别为 `Info`。
- 日志格式：`[YYYY-MM-DD HH:MM:SS | LEVEL | file:line] --> message`。
- panic hook 记录 panic，并继续调用先前的 hook。
- 配置读取/迁移/保存、启动曲库扫描、歌曲读取/播放、音频输出和关键后端错误均记录适当级别。
- logger 在 app native module 注册时、host 开始处理 UI 前初始化；Rust 和 TSX 共用同一初始化入口，重复调用不重复打开文件。初始化需兼容宿主已有 logger，不能覆盖宿主 logger 或让应用启动失败。

## 测试设计

- `src/config.rs` 单元测试：Slint 默认值、TOML 字段与枚举 round-trip、Slint 旧配置中带 `serde(default)` 字段缺失时的兼容、损坏文件回退、旧 NSH Storage 映射、迁移写入失败时保留旧值。
- `src/logger.rs` 单元测试：时间/级别/位置格式、控制台与文件双写、10 MiB 轮转和初始化重复调用。
- UI 持久化测试：Rust 和 TSX 从同一 Config 启动，修改曲目/进度/排序/主题等状态，关闭后重载验证快照；同时验证 Gallery 滚动恢复。
- 渲染检查：Rust 和 TSX 分别在浅色、深色主题下渲染曲库与设置页。
- 完成验证：运行 `cargo fmt --check`、`cargo test --locked`、`nsh check -v` 和相关 Rust/TSX 渲染测试。

## 范围外

不更改音频解码、元信息格式、界面布局和播放算法；不改写 Slint 既有配置的字段名或目录结构。
