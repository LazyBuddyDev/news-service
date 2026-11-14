# news service development

- This repository owns its business API and build/package tests. Do not import desktop implementation or require a sibling workspace checkout. All external integration uses versioned API, staged packages and explicitly supplied host binaries.
- Keep one TASKS.md entry for the whole LazyBuddy modularization requirement, ID LB-20261113-01; this repository's scope is part of that requirement. Operational commits and script cleanup are evidence, not new tasks. Use literal `[x]` for pending and `[√]` for done.
- Add focused unit/contract tests for non-trivial behavior; use fixtures by default. Do not turn a local change into a full real-account acceptance run. Network/identity/permissions and synthetic/real decoding evidence remain separate.
- Prefer CLI, simple existing code and standard libraries. Research difficult integration against official upstream examples before adding abstractions. Keep UI out of the service, and preserve validation at credentials, network, IPC and local-file boundaries.
- Credentials, device identities, signed URLs and media keys never enter commits, command arguments or public errors. Explicit user authorization is required for real account writes. Do not infer rights from a successful login or one episode.
- Use private dependencies and ignored target/tmp output. Clean verified obsolete artifacts promptly, preserving SDKs, live instances, source and unpushed Git history. Do not run a blanket clean against a shared SDK.
- Confirm own-code and third-party licenses before public distribution. Preserve upstream notices; do not invent a license or release identity. Commit/push/transfer/tag/release are separate actions within current authorization.

- 新增功能必须附带该功能的单元测试；日常只运行新增或受影响的测试与必要构建，不默认执行全部单元、真实接口、媒体或GUI回归。仅关联共享边界或失败证据需要时扩大验证，全量验收由用户明确要求。


## 目录用途与命名

路径相对本服务仓库，按需创建；构建脚本、CI、验证驱动和文档示例使用同一约定。

| 用途 | 固定位置 |
| --- | --- |
| Rust编译产物 | 对应Cargo工作区的默认`target/debug/`、`target/release/`，嵌套工作区遵循其自身默认路径 |
| 验证素材 | `tmp/fixtures/<用途>/`，媒体样例为`tmp/fixtures/media/` |
| 隔离配置、WebView用户数据 | `tmp/profiles/<验证入口>/` |
| 临时测试数据 | `tmp/`下带用途前缀的临时目录，收尾删除 |
| 验证报告、截图、录屏 | `tmp/verification-<用途>/` |
| 功能包、运行时包及校验信息 | `target/packages/`，按模块或用途组织；宿主基础测试包约定为其仓库的`target/packages/base/` |
| 维护者工具环境 | `target/tools/<工具>/`，发布工具环境统一为`target/tools/release/` |
| 私有Go/Python工具链 | `target/toolchains/go/`、`target/toolchains/python/` |
| 可选SDK | `target/sdks/<SDK>/`，如`gstreamer` |
| 下载缓存、wheel和安装源 | `target/downloads/`，如`python-wheels/`、`runtime/` |
| 构建缓存 | `target/cache/<工具>/`，如`go/`；Cargo缓存保持默认结构 |

- 使用表明用途的英文小写kebab-case名称，固定用途复用稳定目录名，不按实验阶段、日期或流水号另建同类目录；并行临时实例可加唯一标识，包协议要求的不可变版本目录保留。
- 不新建或恢复`target/qa-*`、`target/verification-*`、`target/release-tools`和混合工具链的`native-player-sdk`目录；移动目录时同步生成脚本、CI、示例与清理路径。Python虚拟环境迁移后验证解释器、入口脚本及依赖。
- `tmp/`与`target/`保持Git忽略，报告和截图不提交；长期结论归docs或既有功能TASKS条目，历史证据路径不当作新入口。工具环境不得存私钥、认证文件或真实用户数据。
- 构建工具归本服务管理，不依赖宿主或相邻仓库的工具链。已安装功能和运行时由宿主在同一Windows用户的`%LOCALAPPDATA%/LazyBuddy/modules/`统一管理，开发版与安装版共用；验证实例使用独立数据目录和凭据scope。
