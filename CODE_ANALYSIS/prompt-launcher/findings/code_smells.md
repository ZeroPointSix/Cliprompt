# 技术债清单：代码异味

1. 总体评价
当前技术债主要集中在前端单文件巨型组件和后端核心模块过度集中，导致可维护性和可扩展性压力偏高；性能方面存在全量扫描和频繁 IO 的潜在风险，但尚未看到立即的崩溃/安全问题。

2. 具体问题列表（按严重程度排序）
- 🟠 重要：前端主界面单文件体量过大，状态、业务逻辑、UI 全部堆在一个组件中，阅读与修改成本高，容易引入回归。位置：`prompt-launcher/src/routes/+page.svelte`。
- 🟠 重要：核心后端模块职责过于集中，包含配置管理、文件 IO、托盘、热键、窗口控制与命令处理，模块耦合偏高。位置：`prompt-launcher/src-tauri/src/lib.rs`。
- 🟡 次要：窗口尺寸/交互参数存在大量硬编码常量，缺少集中管理，后期调参与一致性维护成本高。位置：`prompt-launcher/src/routes/+page.svelte`（例如 `inputHeight` 等，约 148 行）。
- 🟡 次要：UI 层存在大量重复逻辑（复制/状态更新/最近记录）与重复调用模式，增加维护开销。位置：`prompt-launcher/src/routes/+page.svelte`（`copyPrompt`~`copySnippet` 约 551-606 行）。
- 🟡 次要：文件变更监听事件触发后每次全量重新索引，目录大时会造成卡顿或频繁磁盘读。位置：`prompt-launcher/src-tauri/src/lib.rs:816`（`start_watcher` -> `refresh_prompts`）。

3. 改进建议和示例代码
- 组件拆分：将搜索/结果列表/设置/标签编辑/快捷键等拆分为独立 Svelte 组件，并提取共享逻辑到 `src/lib/` 模块或 store。
- 后端拆分：将 `lib.rs` 中的「配置」「文件索引」「热键/窗口」「托盘」拆分到独立模块，减少交叉依赖。
- 引入常量集中管理：将 UI 尺寸、定时阈值等集中到 `const` 模块，避免散落硬编码。

示例（前端抽离重复逻辑）：
```ts
async function copyAndMark(prompt: PromptEntry, text: string, okMessage: string) {
  await writeText(text);
  await markRecent(prompt);
  status = okMessage;
}

// 使用示例
await copyAndMark(prompt, prompt.body, "已复制到剪贴板");
```

代码质量指标（观察性结论）
- 未引入自动化指标工具（圈复杂度、可维护性指数、重复率未量化）。
- 风险最高的复杂度集中在 `prompt-launcher/src/routes/+page.svelte` 的搜索/渲染/事件处理逻辑及 `prompt-launcher/src-tauri/src/lib.rs` 的全量刷新流程。
- 前端缺少自动化测试覆盖；后端仅有少量单元测试，缺乏端到端/集成验证。
