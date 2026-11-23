# 新闻服务文档

- [新闻API与生命周期](module-api.md)
- [构建、打包和数据目录](../README.md)
- [开发规范](../AGENTS.md)
- [统一文件放置规范](https://github.com/LazyBuddyDev/LazyBuddy/blob/main/docs/file-layout.md)

业务接口只在本仓库维护；宿主通用协议通过链接引用，交付和验证状态见[TASKS](../TASKS.md)。

## 实现入口

| 职责 | 入口 |
| --- | --- |
| JSON Lines 与请求格式 | [main.rs](../src/main.rs)、[protocol.rs](../src/protocol.rs) |
| 注册来源、抓取和解析 | [feeds.rs](../src/feeds.rs) |
| 派发、偏好和缓存 | [state.rs](../src/state.rs) |
| 失败诊断 | [failure.rs](../src/failure.rs) |
| 可选 HTTP 和离线验证 | [http.rs](../src/http.rs)、[test-news-http.py](../scripts/verify/test-news-http.py) |
| 模块清单及完整包 | [module.json](../module.json)、[package.py](../scripts/package.py) |
