# 新闻服务维护指南

Windows x64 独立新闻服务，负责来源、解析、偏好和成功缓存。包内使用 JSON Lines，可选回环 HTTP 复用业务协议。宿主管理可见页面的刷新和界面。

[实现入口](docs/README.md#实现入口) · [业务 API](docs/module-api.md) · [构建与打包](README.md)。源码、文档、开发方案和构建产物按[统一文件放置规范](https://github.com/LazyBuddyDev/LazyBuddy/blob/main/docs/file-layout.md)归位。

- 保留来源 ID、用户偏好和成功缓存的兼容性。批量获取、部分失败、连接方式和回环认证以[新闻 API](docs/module-api.md)为准。
- 来源解析和传输回归使用样例，不访问真实新闻。真实上游结果另行验证。隔离检查显式指定数据目录。
