# news service development

文件归位与命名遵循[文件放置规范](https://github.com/LazyBuddyDev/LazyBuddy/blob/main/docs/file-layout.md)，路径相对本服务仓库；接口入口见[本服务API](docs/module-api.md)。


- This repository owns its business API and build/package tests. Do not import desktop implementation or require a sibling workspace checkout. All external integration uses versioned API, staged packages and explicitly supplied host binaries.
- TASKS.md contains only unfinished goals and the current facts, remaining TODOs, owner/version, necessary evidence, blockers and next step needed for handoff. One complete goal has one stable ID; phases are internal steps. Remove completed or fully superseded entries without an archive; compare all remaining goals before merging partial overlaps. Do not create entries for scripts, rules, commits or synchronization alone. Use literal `[x]` for pending and `[√]` only for necessary completed steps inside a current task.
- Add focused unit/contract tests for non-trivial behavior; use fixtures by default. Do not turn a local change into a full real-account acceptance run. Network/identity/permissions and synthetic/real decoding evidence remain separate.
- Prefer CLI, simple existing code and standard libraries. Research difficult integration against official upstream examples before adding abstractions. Keep UI out of the service, and preserve validation at credentials, network, IPC and local-file boundaries.
- Credentials, device identities, signed URLs and media keys never enter commits, command arguments or public errors. Explicit user authorization is required for real account writes. Do not infer rights from a successful login or one episode.
- Use private dependencies and ignored target/tmp output. Clean verified obsolete artifacts promptly, preserving SDKs, live instances, source and unpushed Git history. Do not run a blanket clean against a shared SDK.
- Confirm own-code and third-party licenses before public distribution. Preserve upstream notices; do not invent a license or release identity. Commit/push/transfer/tag/release are separate actions within current authorization.

- 新增功能必须附带该功能的单元测试；日常只运行新增或受影响的测试与必要构建，不默认执行全部单元、真实接口、媒体或GUI回归。仅关联共享边界或失败证据需要时扩大验证，全量验收由用户明确要求。


## 文档维护

- 根README只放产品介绍与快速入口，docs/README维护专题导航，AGENTS只放长期约束，功能进度归TASKS；具体归位按文件放置规范执行，不重复维护目录表。
- 一个主题只保留一份权威说明。宿主维护通用协议，各服务维护自己的业务API，跨仓库使用明确链接，不复制整份文档；说明当前实现；TASKS仅保留未完成任务接手所需证据，长期结论归专题，不维护已完成任务历史。
- 专题文件名使用稳定英文kebab-case，不追加日期、版本或阶段；小规模专题保持平铺。新增、合并或删除文档时同步导航、README和所有有效引用，命令路径相对所属仓库，不写开发机固定路径或凭据。
- 一次性报告、截图和录屏放已忽略的tmp，长期结论归并专题；公开站点和签名机器端点另有用途，不作为普通文档删除或格式化，签名内容保留精确字节。
- 纯文档修改检查链接、示例入口与已发布文件的字节完整性，不默认运行全量功能测试；构建通过、样例通过和真实验收分开报告。
