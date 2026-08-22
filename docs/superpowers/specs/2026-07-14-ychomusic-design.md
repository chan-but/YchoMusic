# YchoMusic 设计文档

## 1. 项目概述

YchoMusic 是一款基于 Tauri + Rust + Svelte 的跨平台本地音乐播放器，主打轻量级、高性能、美观的暗色液态胶囊风格界面。

### 1.1 核心特性

- **播放核心**：纯 Rust 音频解码（symphonia）+ 播放输出（rodio），支持 FLAC/APE/WAV/MP3/OGG/M4A/AAC
- **UI 设计**：液态玻璃/毛玻璃效果、3D 悬浮组件、暗色主题、自定义配色
- **交互体验**：专辑封面滚动列表、歌词悬浮、频谱可视化、全局热键
- **跨平台**：一套代码支持 Windows/macOS/Linux
- **性能**：低内存占用、高效 IO、GPU 加速渲染

### 1.2 参考项目

- YeahMusic：专辑封面滚动效果
- WaveFlow：视觉设计风格
- MusicPlayer2：功能完整性

---

## 2. 技术栈

| 层级 | 技术 | 版本 | 职责 |
|---|---|---|---|
| 桌面壳 | Tauri | 2.x | 窗口管理、IPC、系统托盘 |
| 前端框架 | Svelte | 5.x | UI 组件、路由、响应式状态 |
| 语言 | TypeScript | 5.x | 前端逻辑 |
| 样式 | Tailwind CSS | 4.x | 原子化 CSS、主题系统 |
| 音频解码 | symphonia | 0.12.x | FLAC/APE/WAV/MP3/OGG/M4A/AAC 解码 |
| 音频输出 | rodio | 0.18.x | 播放控制、音量、设备选择 |
| 标签读取 | lofty | 0.18.x | ID3/Vorbis/AAC 标签读写 |
| 持久化 | rusqlite | 0.31.x | SQLite 数据库 |
| 状态管理 | Svelte Stores | - | 前端响应式状态 |

---

## 3. 架构设计

### 3.1 项目结构

```
YchoMusic/
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # Tauri 入口
│   │   ├── commands/             # Tauri Command 模块
│   │   │   ├── audio.rs          # 播放控制
│   │   │   ├── library.rs        # 文件扫描、ID3标签读取
│   │   │   ├── playlist.rs       # 歌单 CRUD
│   │   │   ├── lyrics.rs         # 歌词解析
│   │   │   ├── settings.rs       # 配置读写
│   │   │   └── stats.rs          # 播放统计
│   │   ├── audio/                # 音频引擎
│   │   │   ├── decoder.rs        # symphonia 解码器
│   │   │   ├── output.rs         # rodio 输出
│   │   │   ├── spectrum.rs       # 频谱数据
│   │   │   └── player.rs         # 播放器状态
│   │   ├── db/                   # SQLite 持久化
│   │   │   ├── schema.rs         # 数据库模式
│   │   │   └── connection.rs     # 连接管理
│   │   └── models/               # 数据结构
│   │       ├── track.rs          # 歌曲模型
│   │       ├── playlist.rs       # 歌单模型
│   │       └── settings.rs       # 设置模型
│   └── Cargo.toml
├── src/                          # Svelte 前端
│   ├── App.svelte                # 根组件
│   ├── main.ts                   # 入口文件
│   ├── routes/                   # 页面路由
│   │   ├── Home.svelte           # 首页
│   │   └── Player.svelte         # 播放页
│   ├── components/               # UI 组件
│   │   ├── AlbumList/            # 滚动专辑列表
│   │   │   ├── AlbumList.svelte
│   │   │   └── AlbumItem.svelte
│   │   ├── ProgressBar/          # 悬浮进度条
│   │   │   └── ProgressBar.svelte
│   │   ├── Lyrics/               # 歌词面板
│   │   │   └── Lyrics.svelte
│   │   ├── Capsule/              # 悬浮胶囊组件
│   │   │   └── Capsule.svelte
│   │   ├── Spectrum/             # 频谱可视化
│   │   │   └── Spectrum.svelte
│   │   ├── Settings/             # 设置面板
│   │   │   └── Settings.svelte
│   │   ├── Stats/                # 统计模块
│   │   │   └── Stats.svelte
│   │   └── PlaylistOverlay/      # 播放列表弹出
│   │       └── PlaylistOverlay.svelte
│   ├── stores/                   # Svelte stores
│   │   ├── player.ts             # 播放器状态
│   │   ├── library.ts            # 音乐库状态
│   │   └── settings.ts           # 设置状态
│   └── styles/                   # 样式
│       ├── themes.ts             # 主题定义
│       └── global.css            # 全局样式
├── index.html
├── package.json
├── svelte.config.js
├── tailwind.config.js
└── vite.config.ts
```

### 3.2 核心数据流

```
Svelte UI  ←→  Tauri Commands (IPC)  ←→  Rust Backend
  ↑                                         ↓
Svelte Stores (响应式)              symphonia (解码) → rodio (输出)
                                         ↓
                                   SQLite (持久化)
```

### 3.3 窗口架构

**单窗口起步**：所有视图在同一个 Tauri 窗口内通过 Svelte 路由切换。桌面歌词在 Phase 3 通过独立透明窗口实现。

---

## 4. 视觉风格与主题系统

### 4.1 设计语言

**核心风格**：暗色液态胶囊

- 背景：深色渐变 + 噪点纹理
- 容器：胶囊形圆角（`border-radius: 24px+`）+ 玻璃拟态
- 阴影：多层柔和阴影模拟深度
- 动画：缓动曲线（`cubic-bezier(0.4, 0, 0.2, 1)`）+ 轻微阻尼感

### 4.2 主题系统

| 主题 | 配色特点 | 默认 |
|---|---|---|
| 液态玻璃 | 深蓝灰底 + 青色高光 + 柔和模糊 | ✓ |
| 黄昏 | 橙紫渐变 + 金色高光 | |
| 黑夜 | 纯黑底 + 白色细边框 + 极简 | |
| 赛博朋克 | 粉紫霓虹 + 青色发光 + 网格背景 | |
| 毛玻璃 | 灰白色调 + 强模糊 + 柔和阴影 | |
| 自定义 | 用户选主色/辅色/强调色 | |

### 4.3 CSS 变量结构

```css
:root {
  --bg-primary: #0d0d12;
  --bg-secondary: #15151d;
  --bg-card: rgba(21, 21, 29, 0.8);
  --text-primary: #f0f0f5;
  --text-secondary: #8888a0;
  --accent: #00d4ff;
  --accent-glow: rgba(0, 212, 255, 0.3);
  --border: rgba(255, 255, 255, 0.08);
  --blur: 20px;
}
```

---

## 5. 播放界面设计

### 5.1 整体布局

```
┌─────────────────────────────────────────────────────┐
│  [收起箭头↑]              播放界面                     │
├──────────┬──────────────────────────────────────────┤
│ 左侧专辑  │                                         │
│ 滚动列表  │           右侧：歌词展示                  │
│  (可收起) │           - 点击跳转进度                 │
│          │           - 高亮当前行                    │
│          │           - 右下角定位图标                │
│          │                                         │
├──────────┴──────────────────────────────────────────┤
│           ← 底部悬浮进度条（横跨整个底部，居中）→      │
│  [模式] [上一首] [播放/暂停] [下一首] [列表] [音量] [收藏] 时长 │
└─────────────────────────────────────────────────────┘
```

### 5.2 左侧专辑滚动列表

**三种模式**：

| 模式 | 描述 |
|---|---|
| 纯文字 | 列表仅显示歌名、艺人、时长 |
| 封面+文字 | 左侧小封面 + 右侧歌名/艺人/时长 |
| 大封面滚动 | YeahMusic 风格，封面放大，滚动时缩放+景深效果 |

**滚动交互**：
- 鼠标滚轮滚动切换歌曲
- 选中项：放大（`scale: 1.1`）+ 清晰（`filter: blur(0)`）
- 未选中项：缩小（`scale: 0.85`）+ 变暗（`opacity: 0.4`）
- 阻尼效果：`transition: transform 0.3s cubic-bezier(0.25, 0.1, 0.25, 1)`
- 立体跟随：鼠标移动时封面轻微倾斜（`rotateX/Y` 基于鼠标位置）

**收起行为**：
- 鼠标悬浮收起按钮 → 展开列表
- 10 秒无操作 → 自动收回
- 收回时歌词 + 进度条居中显示

### 5.3 底部进度条

**功能区**（从左到右）：
1. 模式选择（随机/单曲循环/列表循环/列表不循环）
2. 上一首
3. 播放/暂停
4. 下一首
5. 展开列表（弹出 overlay）
6. 音量调节
7. 收藏
8. 时长/进度

**展开列表 overlay**：
- 弹出独立小窗口（胶囊形）
- 显示播放历史 → 当前播放 → 待播放
- 随机模式下可拖动排序

**布局实现**：
```css
.ProgressBar {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  width: calc(100% - 40px);
  max-width: 900px;
}
```

### 5.4 右侧歌词

- 最多显示 15 行
- 当前行高亮 + 发光
- 点击任意行跳转到对应时间
- 自动滚动跟随

### 5.5 定位功能

- 右下角定位图标
- 点击将左侧滚动列表定位到当前播放歌曲
- 滚动时有动画，提前两首歌开始减速并伴有阻尼感

---

## 6. 首页设计

### 6.1 布局风格

抛弃传统侧边栏，采用悬浮胶囊组件架构。

```
┌─────────────────────────────────────────────────────┐
│                    YchoMusic                        │
│              (发光logo + 简洁标题)                   │
├─────────────────────────────────────────────────────┤
│                                                     │
│     ┌──────────────┐     ┌──────────────┐          │
│     │    统计       │     │    歌单       │          │
│     │  [播放次数]   │     │  [歌单列表]   │          │
│     │  [播放时长]   │     │  [导入文件夹] │          │
│     └──────────────┘     └──────────────┘          │
│                                                     │
│     ┌──────────────┐     ┌──────────────┐          │
│     │    设置       │     │  音乐标签     │          │
│     │  [常规设置]   │     │  [标签编辑]   │          │
│     │  [外观主题]   │     │  [联网获取]   │          │
│     └──────────────┘     └──────────────┘          │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### 6.2 胶囊组件样式

- 圆角：`border-radius: 20px`
- 玻璃拟态：`backdrop-filter: blur(20px)`
- 悬浮效果：`box-shadow: 0 8px 32px rgba(0,0,0,0.3)`
- 交互：hover 时上浮 + 发光，点击展开

### 6.3 统计模块

**统计规则**：只有完整播放（进度 > 90% 且未手动拖动进度条）才计入

**展示内容**：
- 总播放歌曲数
- 总播放时长（小时:分钟:秒）
- Top 10 播放最多歌曲
- 时间方格（日历热力图）：
  - X 轴：时间（小时）
  - Y 轴：日期
  - 颜色深浅：播放数量
  - 可切换：日/周/月/年视图（最多一年范围）

### 6.4 歌单模块

**核心功能**：
- 导入本地文件夹作为歌单
- 保存歌单配置（路径 + 排序方式）
- 启动时自动重新扫描（保持用户排序）

**排序方式**：
| 排序键 | 顺序/逆序 |
|---|---|
| 专辑 | ✓ |
| 修改时间 | ✓ |
| 歌曲-艺人 | ✓ |
| 艺人-歌曲 | ✓ |
| 时长 | ✓ |

### 6.5 设置模块

**分类**：
- 常规设置（开机自启、语言、更新检查、记忆播放位置、字体、GPU 加速）
- 外观设置（主题选择、频谱可视化开关）
- 歌词设置（内嵌/文件歌词、歌词行数、桌面歌词开关）
- 系统设置（关闭行为、磁盘缓存、全局热键）
- 播放设置（自动播放、音频输出设备）
- 关于（版本、作者）

### 6.6 音乐标签模块

**功能**：
- 读取/编辑 ID3 标签（歌名、作者、专辑、年份、音轨号）
- 读取封面图片
- 读取内嵌歌词
- 联网获取缺失标签（可选）

---

## 7. 音频引擎设计（Rust）

### 7.1 解码器 (decoder.rs)

**支持格式**：FLAC、APE、WAV、MP3、OGG、M4A、AAC

**核心流程**：
```
File Path → symphonia::default::get_probe() → Track → Decoder → AudioBuffer
```

**关键函数**：
```rust
pub struct AudioDecoder {
    decoder: Box<dyn Decoder>,
    reader: MediaSourceStream,
}

impl AudioDecoder {
    pub fn new(path: &Path) -> Result<Self>;
    pub fn decode(&mut self) -> Result<AudioBufferRef>;
    pub fn duration(&self) -> Duration;
    pub fn metadata(&self) -> &Metadata;
}
```

### 7.2 播放器 (player.rs)

**状态管理**：
```rust
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

pub enum PlayMode {
    Normal,       // 列表不循环
    RepeatList,   // 列表循环
    RepeatOne,    // 单曲循环
    Shuffle,      // 随机播放
}

pub struct Player {
    state: PlayState,
    mode: PlayMode,
    current_track: Option<TrackId>,
    playlist: Vec<TrackId>,
    position: Duration,
    volume: f32,          // 0.0 - 1.0
    playback_rate: f32,   // 0.5 - 2.0
}
```

**播放控制**：
- `play()` / `pause()` / `stop()`
- `prev()` / `next()`
- `seek(position: Duration)`
- `set_volume(volume: f32)`
- `set_mode(mode: PlayMode)`

### 7.3 频谱可视化 (spectrum.rs)

**实现方式**：从解码后的音频 buffer 提取 PCM 数据，计算 FFT 获取频谱。

**输出**：每个声道的频率分量数组（128 个频段），通过 Tauri Event 每秒发送 30 次。

---

## 8. 数据持久化（SQLite）

### 8.1 数据库结构

```sql
CREATE TABLE tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    title TEXT,
    artist TEXT,
    album TEXT,
    year INTEGER,
    track_number INTEGER,
    duration INTEGER,  -- ms
    bitrate INTEGER,
    sample_rate INTEGER,
    cover_blob BLOB,
    lyrics TEXT,
    added_at INTEGER,
    modified_at INTEGER
);

CREATE TABLE playlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    sort_key TEXT DEFAULT 'title',
    sort_order TEXT DEFAULT 'asc',
    created_at INTEGER
);

CREATE TABLE playlist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_id INTEGER,
    track_id INTEGER,
    position INTEGER,
    FOREIGN KEY(playlist_id) REFERENCES playlists(id),
    FOREIGN KEY(track_id) REFERENCES tracks(id)
);

CREATE TABLE play_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER,
    played_at INTEGER,
    duration_played INTEGER,  -- ms
    completed BOOLEAN,        -- 是否完整播放
    FOREIGN KEY(track_id) REFERENCES tracks(id)
);

CREATE TABLE favorites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER UNIQUE,
    added_at INTEGER,
    FOREIGN KEY(track_id) REFERENCES tracks(id)
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT
);
```

### 8.2 配置存储

设置项存储在 `settings` 表中，同时缓存到 `settings.json` 文件以便快速读取。

---

## 9. IPC 层设计（Tauri Commands）

### 9.1 音频控制

| Command | 描述 |
|---|---|
| `play()` | 播放当前歌曲 |
| `pause()` | 暂停播放 |
| `stop()` | 停止播放 |
| `prev()` | 上一首 |
| `next()` | 下一首 |
| `seek(position: f64)` | 跳转进度（秒） |
| `set_volume(volume: f32)` | 设置音量 |
| `set_mode(mode: String)` | 设置播放模式 |
| `get_player_state()` | 获取播放器状态 |

### 9.2 音乐库

| Command | 描述 |
|---|---|
| `scan_directory(path: String)` | 扫描目录 |
| `get_tracks(filter: TrackFilter)` | 获取歌曲列表 |
| `get_track_detail(id: i64)` | 获取歌曲详情 |
| `get_playlists()` | 获取歌单列表 |
| `create_playlist(name: String)` | 创建歌单 |
| `add_to_playlist(playlist_id: i64, track_ids: Vec<i64>)` | 添加歌曲到歌单 |

### 9.3 歌词

| Command | 描述 |
|---|---|
| `get_lyrics(track_id: i64)` | 获取歌词 |
| `parse_lrc(content: String)` | 解析 LRC 格式 |

### 9.4 收藏

| Command | 描述 |
|---|---|
| `toggle_favorite(track_id: i64)` | 收藏/取消收藏歌曲 |
| `get_favorites()` | 获取收藏列表 |

### 9.5 统计

| Command | 描述 |
|---|---|
| `get_stats()` | 获取播放统计 |
| `get_play_history(days: i64)` | 获取播放历史 |

### 9.6 设置

| Command | 描述 |
|---|---|
| `get_settings()` | 获取所有设置 |
| `set_setting(key: String, value: String)` | 设置配置项 |

### 9.7 事件通知

| Event | 描述 |
|---|---|
| `player_state_changed` | 播放状态变化 |
| `track_changed` | 歌曲切换 |
| `position_changed` | 播放进度变化 |
| `volume_changed` | 音量变化 |
| `spectrum_data` | 频谱数据（每秒 30 次） |
| `favorite_changed` | 收藏状态变化 |

---

## 10. 缓存系统设计

### 10.1 缓存策略

| 缓存类型 | 缓存内容 | 缓存位置 | 过期策略 |
|---|---|---|---|
| 音频缓存 | 解码后的 PCM 数据 | 磁盘（配置目录/cache/audio） | LRU，达到上限自动清理 |
| 封面缓存 | 解码后的封面图片 | 磁盘（配置目录/cache/covers） | 永不过期，文件修改时刷新 |
| 歌词缓存 | 解析后的歌词数据 | 磁盘（配置目录/cache/lyrics） | 永不过期，文件修改时刷新 |
| 设置缓存 | 设置项 JSON | 内存 + 磁盘（settings.json） | 实时同步 |

### 10.2 缓存配置

| 配置项 | 描述 | 默认值 |
|---|---|---|
| cache_enabled | 是否启用磁盘缓存 | true |
| cache_directory | 缓存目录路径 | 系统默认缓存目录 |
| cache_max_size | 缓存上限（MB） | 500 |
| cache_cleanup_policy | 达到上限时的清理策略 | LRU |

### 10.3 缓存清理

- 达到 `cache_max_size` 时触发清理
- 按 LRU 顺序删除最久未访问的缓存文件
- 保留至少 10% 的缓存空间

---

## 11. 全局热键设计

### 11.1 默认快捷键

| 按键 | 功能 |
|---|---|
| `Ctrl + Space` | 播放/暂停 |
| `Ctrl + Left` | 上一首 |
| `Ctrl + Right` | 下一首 |
| `Ctrl + Up` | 音量增加 |
| `Ctrl + Down` | 音量减少 |
| `Ctrl + S` | 收藏当前歌曲 |
| `Ctrl + F` | 搜索 |
| `Ctrl + L` | 切换歌词显示 |
| `Ctrl + 1-4` | 切换播放模式 |

### 11.2 自定义热键

用户可在设置中修改任意快捷键，冲突检测（同一按键不能绑定多个功能）。

---

## 12. 缺失功能补充

### 12.1 搜索功能
- 全局搜索框（按歌名、艺人、专辑搜索）
- 实时搜索结果展示

### 12.2 播放列表导出/导入
- 导出为 M3U/M3U8/PLS 格式
- 导入 M3U/M3U8/PLS 文件

### 12.3 音频增益（ReplayGain）
- 自动计算音量增益
- 播放时自动调整音量至统一水平

### 12.4 系统托盘
- 最小化到托盘
- 托盘菜单（播放/暂停/上一首/下一首/退出）
- 托盘图标显示播放状态

### 12.5 窗口控制
- 窗口置顶/取消置顶
- 窗口透明度调节

### 12.6 快捷键提示
- 首次打开时显示快捷键提示卡
- 设置中可查看所有快捷键

---

## 13. 开发阶段规划

### Phase 1：播放核心（MVP）
- Rust 音频引擎（symphonia + rodio）
- Tauri 框架搭建
- 基本播放控制（播放/暂停/上一首/下一首/进度条）
- ID3 标签读取
- 歌单导入（文件夹扫描）
- 简单播放界面

### Phase 2：UI 完善
- Svelte 5 完整界面
- 液态玻璃主题系统
- 专辑滚动列表（3D 效果）
- 歌词面板
- 底部进度条（悬浮）
- 首页胶囊组件
- 设置界面
- 统计模块

### Phase 3：高级功能
- 频谱可视化
- 全局热键
- 桌面歌词（独立透明窗口）
- 音乐标签编辑
- 播放列表导出/导入
- 系统托盘
- 音频增益
- 缓存系统

### Phase 4：优化与完善
- 性能优化（内存/CPU）
- 跨平台测试（Windows/macOS/Linux）
- 主题自定义
- 搜索功能
- 错误处理与日志
- 打包发布

---

## 14. 质量保证

### 14.1 测试策略
- 单元测试：Rust 音频引擎、数据库操作
- 集成测试：IPC 通信、播放流程
- E2E 测试：核心用户流程

### 14.2 性能指标
- 内存占用：< 100MB（空闲），< 200MB（播放中）
- 启动时间：< 2 秒
- 音频解码：支持 32bit/384kHz 无损格式

### 14.3 错误处理
- 音频解码失败：优雅降级，记录日志，跳过当前歌曲
- 文件访问失败：提示用户权限问题
- 数据库损坏：自动重建或提示用户

---

## 15. 发布计划

### 15.1 GitHub 开源
- MIT 许可证
- 完整 README（功能介绍、安装指南、开发文档）
- 贡献指南

### 15.2 打包格式
- Windows：MSI / Portable
- macOS：DMG
- Linux：AppImage / DEB / RPM

---

**文档版本**: v1.1  
**创建日期**: 2026-07-14  
**更新日期**: 2026-07-14  
**状态**: 已自审
