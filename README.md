# LazyBuddy News Service

Independent Windows x64 news API for any client, owned by LazyBuddyDev/news-service. No Iced or desktop source dependency. The desktop consumes JSON-lines API v1; an optional authenticated loopback HTTP adapter exposes the same protocol.

```powershell
cargo test --locked
cargo build --release --locked
python -B scripts/package.py --output target/packages
```

The feature ZIP contains its own module.json, executable, README and third-party notices. No desktop checkout or SDK is required. Package output is a fresh directory inside target. Own code is MIT, copyright 2026 Leager. Third-party code retains its original licenses; see LICENSE and notices.

`sources`, `fetch`, `state`, `preferences` and `configure` provide source registration, a maximum four-source batch, saved successful cache and per-service settings. There is no background refresh timer. Failed refreshes keep successful cached stories; malformed local state is not overwritten. Data defaults to %LOCALAPPDATA%/LazyBuddyNews, or LAZYBUDDY_DATA_DIR/services/news; LAZYBUDDY_NEWS_DATA_DIR is an explicit override.

Build HTTP with `cargo build --features http --bin lazybuddy-news-http --locked`. Set LAZYBUDDY_API_TOKEN to at least 32 random alphanumeric characters. The server binds 127.0.0.1, requires bearer authentication, rejects Origin, limits request bodies and reports busy requests. Test with `python -B scripts/verify/test-news-http.py`; the tests never fetch upstream news.

API fields and lifecycle are in [module API](docs/module-api.md); see [documentation](docs/README.md) for the maintained topics. Ordinary changes run the related unit/contract tests. No real upstream request is needed for source parsing or transport regression. The unchanged news0.1.0 binary is included in the immutable [LazyBuddy 2.0.0 module snapshot](https://github.com/LazyBuddyDev/LazyBuddy/releases/tag/modules-v2.0.0); the signed Web catalog is maintained by the host repository. Future binary releases and catalog publication remain explicit actions.
