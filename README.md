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

## 简介

Cliprompt 是一款轻量级的 Windows 桌面应用，专为提示词管理和快速粘贴设计。通过全局快捷键，你可以在任何应用中快速搜索并粘贴提示词，大幅提升工作效率。

**核心特点**：
- 🚀 **极速启动** - 全局快捷键一键呼出
- 📁 **文件驱动** - 所有提示词存储为本地 `.txt` / `.md` 文件
- 🔍 **智能搜索** - 模糊搜索，支持标签过滤
- ⚡ **自动粘贴** - 选择即粘贴，无需手动复制
- 🏷️ **标签管理** - 通过文件名或文件夹自动识别标签
- 💾 **完全本地** - 数据存储在本地，隐私安全

---

## 功能特性

### 核心功能
- ⌨️ **全局快捷键** - 默认 `Alt+Space` 随时呼出/隐藏窗口
- 🔎 **即时搜索** - 支持标题、标签、内容全文搜索
- 📋 **自动粘贴** - 选择提示词后自动粘贴到当前应用
- 👁️ **实时预览** - 搜索结果显示匹配的代码片段
- 🔄 **热重载** - 文件变更自动刷新，无需重启

### 高级功能
- ⭐ **收藏功能** - `Ctrl+Shift+F` 收藏/取消，`Ctrl+Shift+G` 过滤收藏
- 🕒 **最近使用** - `Ctrl+Shift+E` 查看最近使用，`Ctrl+Shift+R` 清空记录
- 🏷️ **标签系统** - 支持 `#标签` 和 `[标签]` 语法，右键管理标签
- 🖱️ **右键菜单** - 打开源文件、添加/删除标签、删除提示词
- 🎯 **托盘驻留** - 系统托盘显示/隐藏/退出
- 🔧 **开机自启** - 可选配置开机启动

---

## 快速开始

### 安装

1. 前往 [Releases](https://github.com/ZeroPointSix/Cliprompt/releases) 页面
2. 下载最新版本的 `.msi` 安装包
3. 双击安装包完成安装

### 首次使用

1. 安装后程序会自动启动（或手动启动）
2. 默认快捷键 `Alt+Space` 呼出窗口
3. 首次运行会在 `文档/PromptLauncher/Prompts` 创建示例文件
4. 开始搜索并使用提示词！

---

## 使用说明

### 基本操作

| 操作 | 说明 |
|------|------|
| `Alt+Space` | 显示/隐藏窗口 |
| `输入文字` | 搜索提示词 |
| `Enter` | 粘贴选中的提示词 |
| `双击` | 粘贴提示词 |
| `右键` | 打开提示词源文件 |
| `Esc` | 隐藏窗口 |

### 高级快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Shift+F` | 收藏/取消收藏 |
| `Ctrl+Shift+G` | 过滤收藏 |
| `Ctrl+Shift+E` | 查看最近使用 |
| `Ctrl+Shift+R` | 清空最近记录 |

### 标签使用

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

### 右键菜单

- **打开文件** - 使用默认编辑器打开提示词
- **添加标签** - 为提示词添加自定义标签
- **移除标签** - 删除提示词的标签
- **删除** - 删除提示词文件

---

## 开发指南

### 技术栈

- **前端**: Svelte 5 + SvelteKit + Vite
- **后端**: Rust + Tauri 2
- **UI**: 原生 CSS
- **构建**: GitHub Actions

### 本地开发

**前置要求**：
- Node.js 20+
- Rust 工具链
- Tauri 依赖（Windows）

**开发步骤**：

```bash
# 1. 克隆仓库
git clone https://github.com/ZeroPointSix/Cliprompt.git
cd Cliprompt/prompt-launcher

# 2. 安装前端依赖
npm install

# 3. 启动开发服务器
npm run tauri dev
```

### 构建发布

```bash
# 构建 Windows 安装包
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

---

## 数据存储

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

## 常见问题

### Q: 如何修改默认快捷键？
A: 在设置页面可以自定义全局快捷键。

### Q: 提示词文件存在哪里？
A: 默认在 `文档/PromptLauncher/Prompts`，可在设置中修改路径。

### Q: 支持其他文件格式吗？
A: 目前仅支持 `.txt` 和 `.md` 格式。

### Q: 可以云同步吗？
A: 提示词文件夹可以放在 OneDrive/Dropbox 等云盘目录实现同步。

---

## 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

---

## 致谢

- [Tauri](https://tauri.app/) - 强大的桌面应用框架
- [Svelte](https://svelte.dev/) - 优雅的前端框架
- 所有贡献者和用户的支持

---

<div align="center">
  如果觉得项目不错，请给个 ⭐ Star 支持一下！
</div>
