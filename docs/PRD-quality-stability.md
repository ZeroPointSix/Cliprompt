# 功能 PRD：架构治理与可维护性基线（工程治理）

|**文档版本**|**V1.0**|
|---|---|
|**功能名称**|架构治理与可维护性基线（工程治理）|
|**所属产品**|Prompt Launcher|
|**目标平台**|Windows (Tauri Desktop)|
|**优先级**|P0|
|**负责人**|待定|
|**创建日期**|2026-01-31|
|**关联分析**|`CODE_ANALYSIS/prompt-launcher/architecture/`、`CODE_ANALYSIS/prompt-launcher/findings/`|

---

## 1. 背景与问题

当前工程问题集中在架构与可维护性：

- **职责过度集中**：前端主页面与后端 `lib.rs` 责任过度集中，修改成本高、回归风险高。
- **分层不一致**：部分功能有 usecase 层，部分直接调用模块，架构方向不清晰。
- **契约分散**：命令与事件名散落，容易出现变更漂移。

这些问题不影响当前功能正确性，但会显著拖慢迭代速度、引入稳定性风险。

## 2. 目标与非目标

### 2.1 目标
- 建立清晰的 **UI ↔ Backend 契约层**，减少字符串漂移。
- 将 UI 与 Backend 的“上帝模块”拆解为可维护单元。
- 明确 prompts 领域的分层策略并统一落地。
- 增加最小化的架构回归测试覆盖。

### 2.2 非目标
- 不引入新用户功能。
- 不进行跨平台支持扩展。
- 不引入复杂 UI 改版。
- 本期不做性能优化或索引策略重构。

## 3. 用户故事

- 作为开发者，我希望 prompts 相关操作有统一入口，避免分层混乱。
- 作为开发者，我希望 UI 与 backend 的改动范围清晰，减少改一处坏多处的风险。

## 4. 需求范围

### 4.1 契约与常量统一（P0）
- 新增 `tauriClient`，集中封装 `invoke`。
- 事件名、窗口名、阈值集中为常量模块，并在 UI/Backend 同步。
- **验收**：`prompt-launcher/src` 中不再出现裸 `invoke("...")` 字符串。

### 4.2 UI 模块拆分（P0）
- 将 `+page.svelte` 逻辑拆分为 store + 组件。
- 保留页面为布局编排层，业务逻辑移出。
- **验收**：`+page.svelte` 仅做布局和事件分发，不含复杂业务逻辑。

### 4.3 Backend 命令与服务拆分（P0）
- 建立 `src-tauri/src/commands/` 目录，迁出命令实现。
- 将 prompt 相关逻辑归并到 `PromptsService`，统一层次。
- **验收**：`lib.rs` 仅负责 wiring 与启动流程。

### 4.4 Prompts 分层策略统一（P0）
- 选择并明确 prompts 领域分层策略（扩展 usecase 或简化为 service）。
- 将创建/索引/搜索等入口统一到同一层次，不再并行存在两种入口。
- **验收**：`docs/DECISIONS.md` 记录该策略，相关调用路径对齐。

### 4.5 测试与验收（P0）
- 新增对应测试用例文档，覆盖契约统一、模块拆分与分层一致性。
- 补充关键路径的最小自动化测试（优先 Rust 单元测试）。
- **验收**：`docs/test-cases/TC-quality-stability.md` 覆盖核心改动场景。

## 5. 里程碑与阶段

### Phase 0 - Architecture Decision (0.5 天)
- 明确 prompts 分层策略并记录 ADR。
- 确定 commands/services/stores/constants 的边界。

### Phase 1 - Contract & Constants (1-2 天)
- `tauriClient` 与事件常量统一。
- 调用点迁移，完成契约收敛。

### Phase 2 - Backend Split (2-4 天)
- 抽出 `commands/` 与 `PromptsService`。
- `lib.rs` 仅保留 wiring 与启动流程。

### Phase 3 - UI Split (2-4 天)
- 拆分 `+page.svelte`，引入 store 与组件层。

## 6. 验收标准

- `invoke` 与事件名不再散落，均经 `tauriClient`/常量模块引用。
- `+page.svelte` 明显减重且可读。
- `lib.rs` 仅保留启动与 wiring 逻辑。
- prompts 分层策略统一且在 `docs/DECISIONS.md` 有记录。
- 测试用例可重复执行且结果可对比。

## 7. 影响范围与依赖

- 前端：`prompt-launcher/src/routes/+page.svelte`、`prompt-launcher/src/lib/`。
- 后端：`prompt-launcher/src-tauri/src/lib.rs`、`prompt-launcher/src-tauri/src/prompts.rs`。
- 文档：`docs/`、`docs/test-cases/`。

## 8. 风险与对策

| 风险 | 影响 | 对策 |
| --- | --- | --- |
| 拆分过程引发回归 | 功能稳定性下降 | 引入阶段性测试清单与回归用例 |
| 架构一致性分歧 | 反复返工 | 在评审中明确 PromptsService 方向 |

---

## 9. 版本记录

- V1.0: 初版需求定义，覆盖工程治理、架构一致性与测试要求。
