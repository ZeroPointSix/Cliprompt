# Cliprompt

<div align="center">
  <img src="prompt-launcher/src-tauri/icons/128x128.png" alt="Cliprompt Logo" width="128" height="128">

  <p><strong>极简高效的 Windows 提示词启动器</strong></p>

  <p>
    <a href="#功能特性">功能特性</a> •
    <a href="#快速开始">快速开始</a> •
    <a href="#使用说明">使用说明</a> •
    <a href="#开发指南">开发指南</a>
  </p>
</div>

---

## 🚀 简介

Cliprompt 是一款面向提示词重度使用者的 Windows 桌面启动器，专注于“找得快、贴得快、管得清”。用一个全局快捷键，你就能在任何应用里秒速调出窗口，搜索并粘贴提示词，让工作流更顺滑、更专注。

**你将获得：**
- ⚡ 即刻呼出与极速搜索，让提示词触手可及
- 🧩 选中即粘贴，减少复制、切窗与中断
- 🔒 全量本地存储，数据不出本机

---

## 🎯 为什么选择 Cliprompt？

| 传统方式 | Cliprompt |
| --- | --- |
| 提示词分散在各处 | 统一目录集中管理，随时调用 |
| 复制粘贴繁琐 | 选中即粘贴，动作更少 |
| 搜索慢、定位难 | 模糊搜索 + 标签过滤更精准 |
| 难以复用与归类 | 文件名/文件夹自动生成标签 |
| 担心隐私泄露 | 本地优先，不联网更安心 |

---

## 🧭 适用场景

- 📝 写作与文案：常用提示词快速复用，减少重复输入
- 🧑‍💻 研发与测试：技术 Prompt、命令片段随叫随到
- 🧾 客服与运营：标准回复模板统一管理、一键粘贴
- 📚 个人知识库：把高频内容变成随手可用的素材库

---

## ✨ 功能特性

### 核心功能

- ⌨️ **全局快捷键** - 默认 `Alt+Space` 随时呼出/隐藏窗口
- 🔎 **即时搜索** - 标题、标签、内容全文一键检索
- 📋 **自动粘贴** - 选择后直接粘贴到当前应用
- 👁️ **实时预览** - 搜索结果展示匹配片段
- 🔄 **热重载** - 文件变更自动刷新，无需重启

### 高级功能

- ⭐ **收藏功能** - `Ctrl+Shift+F` 收藏/取消，`Ctrl+Shift+G` 过滤收藏
- 🕒 **最近使用** - `Ctrl+Shift+E` 查看最近使用，`Ctrl+Shift+R` 清空记录
- 🏷️ **标签系统** - 支持 `#标签` 和 `[标签]` 语法，右键管理标签
- 🖱️ **右键菜单** - 打开源文件、添加/删除标签、删除提示词
- 🎯 **托盘驻留** - 系统托盘显示/隐藏/退出
- 🔧 **开机自启** - 可选配置开机启动
- ⚙️ **设置页面** - 快捷键、自动粘贴等配置一处管理

---

## 📺 使用说明

### 基本操作

| 操作 | 说明 |
| --- | --- |
| `Alt+Space` | 显示/隐藏窗口 |
| `输入文字` | 搜索提示词 |
| `Enter` | 粘贴选中的提示词 |
| `双击` | 粘贴提示词 |
| `右键` | 打开提示词源文件 |
| `Esc` | 隐藏窗口 |

### 快捷键速览

| 快捷键 | 功能 |
| --- | --- |
| `Ctrl+Shift+F` | 收藏/取消收藏 |
| `Ctrl+Shift+G` | 过滤收藏 |
| `Ctrl+Shift+E` | 查看最近使用 |
| `Ctrl+Shift+R` | 清空最近记录 |

### 标签与搜索

**方式一：文件名标签**
```
邮件回复模板 #email #work.txt
```

**方式二：文件夹标签**
```
Work/
  └── 邮件回复模板.txt  （自动带 #work 标签）
```

**搜索标签**
```
#email          # 搜索包含 email 标签的提示词
#work #email    # 搜索同时包含两个标签的提示词
```

---

## 🚀 快速开始

### 安装

1. 前往 [Releases](https://github.com/ZeroPointSix/Cliprompt/releases) 页面
2. 下载最新版本的 `.msi` 安装包
3. 双击安装包完成安装

### 首次使用

1. 安装后程序会自动启动（或手动启动）
2. 默认快捷键 `Alt+Space` 呼出窗口
3. 首次运行会在 `文档/PromptLauncher/Prompts` 创建示例文件
4. 开始搜索并使用提示词

---

## 🛠 技术栈

- **前端**: Svelte 5 + SvelteKit + Vite
- **后端**: Rust + Tauri 2
- **UI**: 原生 CSS
- **构建**: GitHub Actions

---

## 📁 项目结构

```
prompt-launcher/
  src/                 # SvelteKit 前端
  src-tauri/           # Rust/Tauri 后端
  static/              # 静态资源
docs/                  # 文档与计划
CODE_ANALYSIS/         # 架构与代码分析材料
```

---

## 💾 数据存储

### 提示词文件
- **默认位置**: `C:\Users\你的用户名\Documents\PromptLauncher\Prompts`
- **支持格式**: `.txt` 和 `.md`
- **文件名即标题**（不含扩展名）

### 配置文件
- **位置**: Tauri AppConfig 目录的 `config.json`
- **内容**: 快捷键、自动粘贴、收藏列表等

### 标签元数据
- **位置**: 提示词目录下的 `.tags-meta.json`
- **内容**: 自定义标签关联

---

## 🧑‍💻 开发指南

### 环境准备

- Node.js 20+
- Rust 工具链
- Windows Tauri 依赖

### 本地开发

```bash
cd prompt-launcher
npm install
npm run tauri dev
```

### 代码检查

```bash
npm run check
```

### 构建发布

```bash
npm run tauri build
```

构建产物位于 `prompt-launcher/src-tauri/target/release/bundle/`。

---

## ❓ 常见问题

### Q: 如何修改默认快捷键？
A: 在设置页面可以自定义全局快捷键。

### Q: 提示词文件存在哪里？
A: 默认在 `文档/PromptLauncher/Prompts`，可在设置中修改路径。

### Q: 支持其他文件格式吗？
A: 目前仅支持 `.txt` 和 `.md` 格式。

### Q: 可以云同步吗？
A: 提示词文件夹可以放在 OneDrive/Dropbox 等云盘目录实现同步。

---

## 📝 更新日志

请查看 `docs/DEVLOG.md` 了解版本迭代与变更记录。

---

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

---

## 🙏 致谢

- [Tauri](https://tauri.app/) - 强大的桌面应用框架
- [Svelte](https://svelte.dev/) - 优雅的前端框架
- 所有贡献者和用户的支持

---

<div align="center">
  如果觉得项目不错，请给个 ⭐ Star 支持一下！
</div>
