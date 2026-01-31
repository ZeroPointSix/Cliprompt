# 技术债清单：反模式

1. 总体评价
反模式以“硬编码常量/字符串”和“职责集中”为主，短期可运行但会逐步增加维护成本与缺陷概率。

2. 具体问题列表（按严重程度排序）
- 🟠 重要：前端和后端均出现“上帝模块/上帝组件”趋势，职责与状态高度集中，违反单一职责原则。位置：`prompt-launcher/src/routes/+page.svelte`，`prompt-launcher/src-tauri/src/lib.rs`。
- 🟡 次要：事件名、窗口名、提示文案等均为字符串常量散落，缺乏集中约束，易出现拼写错误或难以统一修改。位置：`prompt-launcher/src-tauri/src/lib.rs:37`（`EVENT_LAUNCHER_SHOWN`）、`prompt-launcher/src/routes/+page.svelte`（字符串事件名）。
- 🟡 次要：大量硬编码数值（窗口尺寸、动画/延迟阈值、分页限制等），建议集中为配置/常量。位置：`prompt-launcher/src/routes/+page.svelte:148` 等。
- 🟡 次要：UI 中存在大量调试日志直接进入生产构建，属于“日志噪音”反模式，可能影响性能与用户隐私。位置：`prompt-launcher/src/routes/+page.svelte:173`、`prompt-launcher/src/routes/+page.svelte:507` 等。

3. 改进建议和示例代码
- 建立 `constants.ts` 统一管理事件名、窗口名、时间阈值与尺寸。
- 使用轻量 logger 并在生产构建中降级或关闭调试日志。

示例（事件/窗口常量集中管理）：
```ts
// src/lib/constants.ts
export const EVENTS = {
  LAUNCHER_SHOWN: "launcher-shown",
  PROMPTS_UPDATED: "prompts-updated"
};

export const UI = {
  WINDOW_WIDTH: 760,
  INPUT_HEIGHT: 56
};
```
