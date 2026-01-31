# 技术债清单：优先级 Backlog

1. 总体评价
技术债以“可维护性与性能风险”为主，建议优先处理前端巨型组件与后端全量刷新逻辑，其余为中低优先级的结构性优化。

2. 具体问题列表（按严重程度排序）
- P0：拆分前端主组件，降低单文件复杂度与回归风险。位置：`prompt-launcher/src/routes/+page.svelte`。
- P1：为文件监听增加去抖/增量刷新机制，避免高频全量扫描。位置：`prompt-launcher/src-tauri/src/lib.rs:816`。
- P1：提取事件名/窗口名/阈值常量，减少硬编码与隐式耦合。位置：`prompt-launcher/src/routes/+page.svelte`、`prompt-launcher/src-tauri/src/lib.rs:37`。
- P2：收敛 UI 日志输出，避免生产环境噪音与性能损耗。位置：`prompt-launcher/src/routes/+page.svelte:173`、`prompt-launcher/src/routes/+page.svelte:507`。
- P2：抽象复制/状态更新通用逻辑，消除重复代码。位置：`prompt-launcher/src/routes/+page.svelte:551`。
- P2：补齐前端测试或最小化的交互用例，降低 UI 复杂逻辑回归风险。

3. 改进建议和示例代码
示例（文件监听去抖）：
```rust
// 伪代码：事件节流，减少全量索引
let mut pending = false;
let mut last_run = Instant::now();

fn on_fs_event() {
  if pending { return; }
  pending = true;
  let now = Instant::now();
  if now.duration_since(last_run) > Duration::from_millis(200) {
    refresh_prompts(...);
    last_run = now;
    pending = false;
  }
}
```
