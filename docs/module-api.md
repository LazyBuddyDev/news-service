# 新闻服务 API

API v1，Windows x64。服务独立提供新闻来源、解析、偏好和成功缓存，不依赖宿主GUI或相邻工作区。通用包与JSON Lines约定见[宿主模块契约](https://github.com/LazyBuddyDev/LazyBuddy/blob/main/docs/module-api.md)，构建入口见[README](../README.md)。

## 传输与方法

stdin每行一个UTF-8 JSON对象，stdout仅写对应回复，不混入日志：

```json
{"version":1,"id":"sources-1","method":"sources","args":{}}
```

成功回复为`{version:1,id,ok:true,data}`，失败为`{version:1,id,ok:false,error:{code}}`。单个请求上限64KiB；关闭stdin正常退出。

| 方法 | 参数 | data |
| --- | --- | --- |
| `health` | 无 | `service: "news"`、`api_version: 1` |
| `sources` | 无 | 来源数组：`id`、`name`、`category`、`subtitle` |
| `fetch` | `sources`：1–4个已知来源ID | 各来源的`id`、`ok`、成功`stories`或`error` |
| `state` | 无 | `preferences`与按来源ID索引的成功`cache` |
| `preferences` | 无 | `preferences`，不刷新内容 |
| `configure` | `preferences`对象 | 保存后的`preferences` |

新闻项字段为`title`、`url`、`badge`。成功来源包含`updated`（Unix秒）与`updated_ms`（Unix毫秒），与持久缓存一致。单个来源失败不清空已有成功缓存；顶层成功不代表所有来源成功。

偏好字段包括`auto_refresh`、`refresh_minutes`（1–60）、`followed`、`hidden`、`order`、`default_followed`。ID数组只能包含已知且不重复的来源；`order`将指定来源排在其余来源之前。客户端应先读取偏好，再合并用户修改提交，避免遗漏字段被默认值替代。无法读取或校验本地状态时返回失败，不用默认状态覆盖原文件。

常见错误包括`invalid_request`、`unsupported_version`、`request_too_large`、`unknown_method`、`expected_one_to_four_sources`、`unknown_source`、`invalid_preferences`；每个来源抓取失败使用`source_failed`。客户端须兼容未知错误码，不能据错误类型推断网络访问已成功。

## 状态与生命周期

数据默认位于`%LOCALAPPDATA%/LazyBuddyNews`。`LAZYBUDDY_NEWS_DATA_DIR`显式覆盖，或使用`LAZYBUDDY_DATA_DIR/services/news`。偏好及成功缓存由本服务持久化，界面主题与窗口设置归客户端。

服务没有后台刷新计时器。客户端只为当前可见页面调度，最多四个实际在途来源；旧请求退出前仍计入并发。离开、隐藏或进入设置后停止派发并隔离迟到结果，不用隐藏组件冒充静默。

## 可选 HTTP

构建与启动见[README](../README.md)。适配器监听127.0.0.1，端点为`POST /v1/news/rpc`，请求体使用同一JSON协议。`LAZYBUDDY_API_TOKEN`至少32个随机字母数字字符，调用需`Authorization: Bearer <token>`。拒绝带Origin的请求，限制请求体并拒绝忙碌期间的新事务，不暴露到公网。

针对性传输验证使用`scripts/verify/test-news-http.py`，不会访问真实新闻上游。单元、真实上游与公开分发分别报告，当前进度见[TASKS](../TASKS.md)。

## Connection mode

News package 0.1.1 accepts the child-process environment variable `LAZYBUDDY_NEWS_DIRECT=1` to explicitly disable all proxies. Otherwise the HTTP client follows proxy environment variables and the current Windows user system proxy. The desktop host persists its own switch and restarts only its news connection when changed; no system proxy settings are modified.
