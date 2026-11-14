use std::{
    error::Error,
    fmt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use ureq::{
    Agent,
    http::Uri,
    tls::{RootCerts, TlsConfig, TlsProvider},
};

use crate::failure::{DataError, Detail};

pub const LIMIT: usize = 30;
pub const ZHIHU_SOURCE_ID: &str = "zhihu-hot-search";
type FeedResult = Result<Vec<Story>, Box<dyn Error>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Badge {
    New,
    Hot,
    Boiling,
    Explosive,
    Meme,
    Exclusive,
    Recommended,
    FirstRelease,
}

impl Badge {
    fn from_weibo(label: &str) -> Option<Self> {
        match label {
            "新" => Some(Self::New),
            "热" => Some(Self::Hot),
            "沸" => Some(Self::Boiling),
            "爆" => Some(Self::Explosive),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Story {
    pub title: Vec<u16>,
    pub url: Vec<u16>,
    pub badge: Option<Badge>,
}

impl Story {
    pub fn parse(title: Option<String>, url: Option<String>) -> Option<Self> {
        let title = title?.replace(['\r', '\n', '\0'], " ");
        let title = title.trim();
        let url = url?;
        let url = url.trim();
        let uri: Uri = url.parse().ok()?;
        if title.is_empty()
            || !matches!(uri.scheme_str(), Some("https" | "http"))
            || uri.host().is_none()
        {
            return None;
        }
        Some(Self {
            title: wide(title),
            url: wide(url),
            badge: None,
        })
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    China,
    World,
    Tech,
    Finance,
    Sports,
}

impl Category {
    pub const ALL: [Self; 5] = [
        Self::China,
        Self::World,
        Self::Tech,
        Self::Finance,
        Self::Sports,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::China => "国内",
            Self::World => "国际",
            Self::Tech => "科技",
            Self::Finance => "财经",
            Self::Sports => "体育",
        }
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Hot,
    Live,
}

enum Request {
    Direct {
        endpoint: &'static str,
        referer: Option<&'static str>,
        accept: &'static str,
        parse: fn(&str) -> FeedResult,
    },
    NewsNow,
}

struct Source {
    id: &'static str,
    name: &'static str,
    category: Category,

    subtitle: Option<&'static str>,
    kind: Option<Kind>,
    request: Request,
}

const SOURCES: [Source; 53] = [
    Source {
        id: "bilibili-hot-search",
        name: "哔哩哔哩热搜",
        category: Category::China,

        subtitle: None,
        kind: None,
        request: Request::Direct {
            endpoint: "https://s.search.bilibili.com/main/hotword?limit=30",
            referer: None,
            accept: "application/json",
            parse: parse_hot_search,
        },
    },
    Source {
        id: "weibo-hot-search",
        name: "微博热搜",
        category: Category::China,

        subtitle: None,
        kind: None,
        request: Request::Direct {
            endpoint: "https://weibo.com/ajax/side/hotSearch?x=1",
            referer: Some("https://weibo.com/"),
            accept: "application/json",
            parse: parse_weibo,
        },
    },
    Source {
        id: ZHIHU_SOURCE_ID,
        name: "知乎热榜",
        category: Category::China,

        subtitle: None,
        kind: None,
        request: Request::Direct {
            endpoint: "https://www.zhihu.com/api/v3/feed/topstory/hot-list-web?limit=30&desktop=true",
            referer: Some("https://www.zhihu.com/"),
            accept: "application/json",
            parse: parse_zhihu,
        },
    },
    Source {
        id: "douyin-hot-search",
        name: "抖音热搜",
        category: Category::China,

        subtitle: None,
        kind: None,
        request: Request::Direct {
            endpoint: "https://www.iesdouyin.com/web/api/v2/hotsearch/billboard/word/",
            referer: Some("https://www.douyin.com/"),
            accept: "application/json",
            parse: parse_douyin,
        },
    },
    Source {
        id: "xiaohongshu-hot-search",
        name: "小红书热榜",
        category: Category::China,

        subtitle: None,
        kind: None,
        request: Request::Direct {
            endpoint: "https://tophub.today/n/L4MdA5ldxD",
            referer: None,
            accept: "text/html",
            parse: parse_xiaohongshu,
        },
    },
    Source {
        id: "wallstreetcn-quick",
        name: "华尔街见闻",
        category: Category::Finance,

        subtitle: None,
        kind: None,
        request: Request::Direct {
            endpoint: "https://api-one.wallstcn.com/apiv1/content/lives?channel=global-channel&limit=30",
            referer: None,
            accept: "application/json",
            parse: parse_wallstreet,
        },
    },
    Source {
        id: "jin10-flash",
        name: "金十快讯",
        category: Category::Finance,

        subtitle: None,
        kind: None,
        request: Request::Direct {
            endpoint: "https://www.jin10.com/flash_newest.js",
            referer: None,
            accept: "text/javascript",
            parse: parse_jin10,
        },
    },
    Source {
        id: "v2ex-share",
        name: "V2EX",
        category: Category::Tech,
        subtitle: Some("最新分享"),
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "zaobao",
        name: "联合早报",
        category: Category::World,
        subtitle: None,
        kind: Some(Kind::Live),
        request: Request::NewsNow,
    },
    Source {
        id: "coolapk",
        name: "酷安",
        category: Category::Tech,
        subtitle: Some("今日最热"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "mktnews-flash",
        name: "MKTNews",
        category: Category::Finance,
        subtitle: Some("快讯"),
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "wallstreetcn-news",
        name: "华尔街见闻",
        category: Category::Finance,
        subtitle: Some("最新"),
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "wallstreetcn-hot",
        name: "华尔街见闻",
        category: Category::Finance,
        subtitle: Some("最热"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "36kr-quick",
        name: "36氪",
        category: Category::Tech,
        subtitle: Some("快讯"),
        kind: Some(Kind::Live),
        request: Request::Direct {
            endpoint: "https://www.36kr.com/newsflashes",
            referer: None,
            accept: "text/html",
            parse: parse_36kr_quick,
        },
    },
    Source {
        id: "36kr-renqi",
        name: "36氪",
        category: Category::Tech,
        subtitle: Some("人气榜"),
        kind: Some(Kind::Hot),
        request: Request::Direct {
            endpoint: "https://gateway.36kr.com/api/mis/nav/home/nav/rank/hot",
            referer: None,
            accept: "application/json",
            parse: parse_36kr_rank,
        },
    },
    Source {
        id: "hupu",
        name: "虎扑",
        category: Category::Sports,
        subtitle: Some("主干道热帖"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "dongqiudi",
        name: "懂球帝",
        category: Category::Sports,
        subtitle: Some("头条"),
        kind: Some(Kind::Live),
        request: Request::NewsNow,
    },
    Source {
        id: "aihot",
        name: "AIHOT",
        category: Category::Tech,
        subtitle: None,
        kind: Some(Kind::Live),
        request: Request::NewsNow,
    },
    Source {
        id: "tieba",
        name: "百度贴吧",
        category: Category::China,
        subtitle: Some("热议"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "toutiao",
        name: "今日头条",
        category: Category::China,
        subtitle: None,
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "ithome",
        name: "IT之家",
        category: Category::Tech,
        subtitle: None,
        kind: Some(Kind::Live),
        request: Request::NewsNow,
    },
    Source {
        id: "thepaper",
        name: "澎湃新闻",
        category: Category::China,
        subtitle: Some("热榜"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "sputniknewscn",
        name: "卫星通讯社",
        category: Category::World,
        subtitle: None,
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "cankaoxiaoxi",
        name: "参考消息",
        category: Category::World,
        subtitle: None,
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "pcbeta-windows11",
        name: "远景论坛",
        category: Category::Tech,
        subtitle: Some("Win11"),
        kind: Some(Kind::Live),
        request: Request::NewsNow,
    },
    Source {
        id: "cls-telegraph",
        name: "财联社",
        category: Category::Finance,
        subtitle: Some("电报"),
        kind: Some(Kind::Live),
        request: Request::NewsNow,
    },
    Source {
        id: "cls-depth",
        name: "财联社",
        category: Category::Finance,
        subtitle: Some("深度"),
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "cls-hot",
        name: "财联社",
        category: Category::Finance,
        subtitle: Some("热门"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "xueqiu-hotstock",
        name: "雪球",
        category: Category::Finance,
        subtitle: Some("热门股票"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "gelonghui",
        name: "格隆汇",
        category: Category::Finance,
        subtitle: Some("事件"),
        kind: Some(Kind::Live),
        request: Request::NewsNow,
    },
    Source {
        id: "fastbull-express",
        name: "法布财经",
        category: Category::Finance,
        subtitle: Some("快讯"),
        kind: Some(Kind::Live),
        request: Request::NewsNow,
    },
    Source {
        id: "fastbull-news",
        name: "法布财经",
        category: Category::Finance,
        subtitle: Some("头条"),
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "solidot",
        name: "Solidot",
        category: Category::Tech,
        subtitle: None,
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "hackernews",
        name: "Hacker News",
        category: Category::Tech,
        subtitle: None,
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "producthunt",
        name: "Product Hunt",
        category: Category::Tech,
        subtitle: None,
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "github-trending-today",
        name: "Github",
        category: Category::Tech,
        subtitle: Some("Today"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "bilibili-hot-video",
        name: "哔哩哔哩",
        category: Category::China,
        subtitle: Some("热门视频"),
        kind: Some(Kind::Hot),
        request: Request::Direct {
            endpoint: "https://api.bilibili.com/x/web-interface/popular",
            referer: Some("https://www.bilibili.com/"),
            accept: "application/json",
            parse: parse_bilibili_videos,
        },
    },
    Source {
        id: "bilibili-ranking",
        name: "哔哩哔哩",
        category: Category::China,
        subtitle: Some("排行榜"),
        kind: Some(Kind::Hot),
        request: Request::Direct {
            endpoint: "https://api.bilibili.com/x/web-interface/ranking/v2",
            referer: Some("https://www.bilibili.com/"),
            accept: "application/json",
            parse: parse_bilibili_videos,
        },
    },
    Source {
        id: "kuaishou",
        name: "快手",
        category: Category::China,
        subtitle: None,
        kind: Some(Kind::Hot),
        request: Request::Direct {
            endpoint: "https://www.kuaishou.com/?isHome=1",
            referer: None,
            accept: "text/html",
            parse: parse_kuaishou,
        },
    },
    Source {
        id: "kaopu",
        name: "靠谱新闻",
        category: Category::World,
        subtitle: None,
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "baidu",
        name: "百度热搜",
        category: Category::China,
        subtitle: None,
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "nowcoder",
        name: "牛客",
        category: Category::China,
        subtitle: None,
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "sspai",
        name: "少数派",
        category: Category::Tech,
        subtitle: None,
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "juejin",
        name: "稀土掘金",
        category: Category::Tech,
        subtitle: None,
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "ifeng",
        name: "凤凰网",
        category: Category::China,
        subtitle: Some("热点资讯"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "chongbuluo-latest",
        name: "虫部落",
        category: Category::China,
        subtitle: Some("最新"),
        kind: None,
        request: Request::NewsNow,
    },
    Source {
        id: "chongbuluo-hot",
        name: "虫部落",
        category: Category::China,
        subtitle: Some("最热"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "douban",
        name: "豆瓣",
        category: Category::China,
        subtitle: Some("热门电影"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "steam",
        name: "Steam",
        category: Category::World,
        subtitle: Some("在线人数"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "tencent-hot",
        name: "腾讯新闻",
        category: Category::China,
        subtitle: Some("综合早报"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "freebuf",
        name: "Freebuf",
        category: Category::China,
        subtitle: Some("网络安全"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "qqvideo-tv-hotsearch",
        name: "腾讯视频",
        category: Category::China,
        subtitle: Some("热搜榜"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
    Source {
        id: "iqiyi-hot-ranklist",
        name: "爱奇艺",
        category: Category::China,
        subtitle: Some("热播榜"),
        kind: Some(Kind::Hot),
        request: Request::NewsNow,
    },
];

pub const SOURCE_COUNT: usize = SOURCES.len();

pub fn source_id(index: usize) -> &'static str {
    SOURCES[index].id
}

pub fn source_category(index: usize) -> Category {
    SOURCES[index].category
}

pub fn source_name(index: usize) -> &'static str {
    SOURCES[index].name
}

pub fn source_subtitle(index: usize) -> Option<&'static str> {
    let source = &SOURCES[index];
    source.subtitle.or(match source.kind {
        Some(Kind::Hot) => Some("热榜"),
        Some(Kind::Live) => Some("实时"),
        None => None,
    })
}

pub fn client() -> Agent {
    Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .into()
}

#[derive(Debug)]
pub struct FetchError {
    stage: &'static str,
    cause: Box<dyn Error>,
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stage={} {}", self.stage, Detail(self.cause.as_ref()))
    }
}

impl Error for FetchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.cause.as_ref())
    }
}

fn at_stage(stage: &'static str) -> impl FnOnce(Box<dyn Error>) -> FetchError {
    move |cause| FetchError { stage, cause }
}

pub fn fetch(client: &Agent, index: usize) -> Result<Vec<Story>, FetchError> {
    let source = &SOURCES[index];
    let stories = match &source.request {
        Request::Direct {
            endpoint,
            referer,
            accept,
            parse,
        } => {
            let mut response = if source.id == "36kr-renqi" {
                client
                    .post(*endpoint)
                    .header("Accept", *accept)
                    .header("Content-Type", "application/json")
                    .send(r#"{"partner_id":"web","param":{"siteId":1,"platformId":2}}"#)
            } else {
                let mut request = client.get(*endpoint).header("Accept", *accept);
                if source.id == "jin10-flash" {
                    request = request.query("t", unix_time().to_string());
                }
                if let Some(referer) = referer {
                    request = request.header("Referer", *referer);
                }
                request.call()
            }
            .map_err(|error| at_stage("request")(error.into()))?;
            let body = response
                .body_mut()
                .with_config()
                .limit(4 * 1024 * 1024)
                .read_to_string()
                .map_err(|error| at_stage("read_body")(error.into()))?;
            parse(&body).map_err(at_stage("parse"))?
        }
        Request::NewsNow => {
            let base = std::env::var("NEWSFLOAT_NEWSNOW_URL")
                .unwrap_or_else(|_| "https://newsnow.busiyi.world".to_owned());
            let endpoint = format!("{}/api/s", base.trim_end_matches('/'));
            let body = client
                .get(endpoint.as_str())
                .query("id", source.id)
                .header("Accept", "application/json")
                .call()
                .map_err(|error| at_stage("request")(error.into()))?
                .body_mut()
                .with_config()
                .limit(1024 * 1024)
                .read_to_string()
                .map_err(|error| at_stage("read_body")(error.into()))?;
            let response: NewsNowResponse =
                serde_json::from_str(&body).map_err(|error| at_stage("parse")(error.into()))?;
            response
                .items
                .into_iter()
                .filter_map(|item| Story::parse(item.title, item.url))
                .take(LIMIT)
                .collect()
        }
    };
    if stories.is_empty() {
        return Err(at_stage("validate")(DataError::Empty.into()));
    }
    Ok(stories)
}

#[derive(Deserialize)]
struct NewsNowResponse {
    items: Vec<NewsNowItem>,
}

#[derive(Deserialize)]
struct NewsNowItem {
    title: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct BilibiliVideos {
    data: BilibiliVideoList,
}

#[derive(Deserialize)]
struct BilibiliVideoList {
    list: Vec<BilibiliVideo>,
}

#[derive(Deserialize)]
struct BilibiliVideo {
    bvid: String,
    title: String,
}

fn parse_bilibili_videos(body: &str) -> FeedResult {
    let response: BilibiliVideos = serde_json::from_str(body)?;
    Ok(response
        .data
        .list
        .into_iter()
        .filter_map(|video| {
            Story::parse(
                Some(video.title),
                Some(format!("https://www.bilibili.com/video/{}", video.bvid)),
            )
        })
        .take(LIMIT)
        .collect())
}

fn parse_36kr_quick(body: &str) -> FeedResult {
    Ok(parse_36kr_links(body, ".newsflash-item a.item-title"))
}

#[derive(Deserialize)]
struct KrRankResponse {
    data: KrRankData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KrRankData {
    hot_rank_list: Vec<KrRankItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KrRankItem {
    item_id: u64,
    template_material: KrRankMaterial,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KrRankMaterial {
    widget_title: Option<String>,
}

fn parse_36kr_rank(body: &str) -> FeedResult {
    let response: KrRankResponse = serde_json::from_str(body)?;
    Ok(response
        .data
        .hot_rank_list
        .into_iter()
        .filter_map(|item| {
            Story::parse(
                item.template_material.widget_title,
                Some(format!("https://36kr.com/p/{}", item.item_id)),
            )
        })
        .take(LIMIT)
        .collect())
}

fn parse_36kr_links(body: &str, selector: &str) -> Vec<Story> {
    let page = scraper::Html::parse_document(body);
    let links = scraper::Selector::parse(selector).expect("valid 36Kr selector");
    page.select(&links)
        .filter_map(|link| {
            let href = link.value().attr("href")?;
            let url = if href.starts_with('/') {
                format!("https://www.36kr.com{href}")
            } else {
                href.to_owned()
            };
            Story::parse(Some(link.text().collect()), Some(url))
        })
        .take(LIMIT)
        .collect()
}

fn parse_kuaishou(body: &str) -> FeedResult {
    let state = body
        .split_once("window.__APOLLO_STATE__")
        .and_then(|(_, state)| state.trim_start().strip_prefix('='))
        .ok_or(DataError::Missing("hot_list_state"))?;
    let data: serde_json::Value = serde_json::Deserializer::from_str(state)
        .into_iter()
        .next()
        .ok_or(DataError::Missing("initial_state"))??;
    let client = &data["defaultClient"];
    let rank = client["ROOT_QUERY"][r#"visionHotRank({"page":"home"})"#]["id"]
        .as_str()
        .ok_or(DataError::Missing("hot_list_id"))?;
    let items = client[rank]["items"]
        .as_array()
        .ok_or(DataError::Missing("hot_list_items"))?;
    Ok(items
        .iter()
        .filter_map(|entry| {
            let item = &client[entry["id"].as_str()?];
            if item["tagType"].as_str() == Some("置顶") {
                return None;
            }
            let title = item["name"].as_str()?;
            let query = utf8_percent_encode(title, NON_ALPHANUMERIC);
            Story::parse(
                Some(title.to_owned()),
                Some(format!(
                    "https://www.kuaishou.com/search/video?searchKey={query}"
                )),
            )
        })
        .take(LIMIT)
        .collect())
}

#[derive(Deserialize)]
struct HotSearch {
    code: i32,
    #[serde(default)]
    list: Vec<HotWord>,
}

#[derive(Deserialize)]
struct HotWord {
    keyword: Option<String>,
    show_name: Option<String>,
    word_type: Option<u32>,
}

fn parse_hot_search(body: &str) -> FeedResult {
    let hot_search: HotSearch = serde_json::from_str(body)?;
    if hot_search.code != 0 {
        return Err(DataError::UpstreamCode(hot_search.code).into());
    }
    Ok(hot_search
        .list
        .into_iter()
        .filter_map(|word| {
            let keyword = word.keyword?;
            let keyword = keyword.trim();
            if keyword.is_empty() {
                return None;
            }
            let query = utf8_percent_encode(keyword, NON_ALPHANUMERIC);
            let title = word
                .show_name
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| keyword.to_owned());
            let mut story = Story::parse(
                Some(title),
                Some(format!("https://search.bilibili.com/all?keyword={query}")),
            )?;
            story.badge = match word.word_type {
                Some(4) => Some(Badge::New),
                Some(5) => Some(Badge::Hot),
                Some(9) => Some(Badge::Meme),
                Some(12) => Some(Badge::Exclusive),
                _ => None,
            };
            Some(story)
        })
        .take(LIMIT)
        .collect())
}

#[derive(Deserialize)]
struct WeiboResponse {
    ok: i32,
    data: Option<WeiboData>,
}

#[derive(Deserialize)]
struct WeiboData {
    realtime: Vec<WeiboWord>,
}

#[derive(Deserialize)]
struct WeiboWord {
    word: Option<String>,
    word_scheme: Option<String>,
    label_name: Option<String>,
    icon_desc: Option<String>,
}

fn parse_weibo(body: &str) -> FeedResult {
    let response: WeiboResponse = serde_json::from_str(body)?;
    if response.ok != 1 {
        return Err(DataError::UpstreamCode(response.ok).into());
    }
    let data = response.data.ok_or(DataError::Missing("hot_search_list"))?;
    Ok(data
        .realtime
        .into_iter()
        .filter_map(|item| {
            let title = item.word?;
            let keyword = item
                .word_scheme
                .as_deref()
                .filter(|word| !word.is_empty())
                .unwrap_or(&title);
            let query = utf8_percent_encode(keyword, NON_ALPHANUMERIC);
            let url = format!("https://s.weibo.com/weibo?q={query}");
            let mut story = Story::parse(Some(title), Some(url))?;
            story.badge = [item.label_name.as_deref(), item.icon_desc.as_deref()]
                .into_iter()
                .flatten()
                .find_map(Badge::from_weibo);
            Some(story)
        })
        .take(LIMIT)
        .collect())
}

#[derive(Deserialize)]
struct ZhihuResponse {
    data: Vec<ZhihuEntry>,
}

#[derive(Deserialize)]
struct ZhihuEntry {
    target: ZhihuQuestion,
}

#[derive(Deserialize)]
struct ZhihuQuestion {
    title_area: ZhihuTitle,
    link: ZhihuLink,
}

#[derive(Deserialize)]
struct ZhihuTitle {
    text: String,
}

#[derive(Deserialize)]
struct ZhihuLink {
    url: String,
}

pub fn is_zhihu_question(url: &str) -> bool {
    url.strip_prefix("https://www.zhihu.com/question/")
        .is_some_and(|id| !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()))
}

fn parse_zhihu(body: &str) -> FeedResult {
    let response: ZhihuResponse = serde_json::from_str(body)?;
    Ok(response
        .data
        .into_iter()
        .map(|entry| entry.target)
        .filter(|question| is_zhihu_question(&question.link.url))
        .filter_map(|question| {
            Story::parse(Some(question.title_area.text), Some(question.link.url))
        })
        .take(LIMIT)
        .collect())
}

#[derive(Deserialize)]
struct DouyinResponse {
    status_code: i32,
    #[serde(default)]
    word_list: Vec<DouyinWord>,
}

#[derive(Deserialize)]
struct DouyinWord {
    word: String,
    label: Option<u32>,
}

fn parse_douyin(body: &str) -> FeedResult {
    let response: DouyinResponse = serde_json::from_str(body)?;
    if response.status_code != 0 {
        return Err(DataError::UpstreamCode(response.status_code).into());
    }
    Ok(response
        .word_list
        .into_iter()
        .filter_map(|item| {
            let keyword = item.word.trim();
            let url = format!(
                "https://www.douyin.com/search/{}",
                utf8_percent_encode(keyword, NON_ALPHANUMERIC)
            );
            let mut story = Story::parse(Some(item.word), Some(url))?;
            story.badge = match item.label {
                Some(1) => Some(Badge::New),
                Some(2) => Some(Badge::Recommended),
                Some(3) => Some(Badge::Hot),
                Some(4) => Some(Badge::Explosive),
                Some(5) => Some(Badge::FirstRelease),
                _ => None,
            };
            Some(story)
        })
        .take(LIMIT)
        .collect())
}

fn parse_xiaohongshu(body: &str) -> FeedResult {
    let page = scraper::Html::parse_document(body);
    let links = scraper::Selector::parse(
        r#"table.table tr > td:nth-child(2) > a[href^="https://www.xiaohongshu.com/search_result?"]"#,
    ).expect("valid hot-list selector");
    Ok(page
        .select(&links)
        .filter_map(|link| {
            let title = link.text().collect::<String>();
            let url = link.value().attr("href")?.to_owned();
            Story::parse(Some(title), Some(url))
        })
        .take(LIMIT)
        .collect())
}

#[derive(Deserialize)]
struct WallStreetResponse {
    data: WallStreetData,
}

#[derive(Deserialize)]
struct WallStreetData {
    items: Vec<WallStreetItem>,
}

#[derive(Deserialize)]
struct WallStreetItem {
    title: Option<String>,
    content_text: Option<String>,
    uri: Option<String>,
}

fn parse_wallstreet(body: &str) -> FeedResult {
    let response: WallStreetResponse = serde_json::from_str(body)?;
    Ok(response
        .data
        .items
        .into_iter()
        .filter_map(|item| {
            let title = item
                .title
                .filter(|title| !title.trim().is_empty())
                .or(item.content_text);
            Story::parse(title, item.uri)
        })
        .take(LIMIT)
        .collect())
}

#[derive(Deserialize)]
struct Jin10Item {
    id: String,
    channel: Option<Vec<u32>>,
    data: Jin10Data,
}

#[derive(Deserialize)]
struct Jin10Data {
    title: Option<String>,
    content: Option<String>,
}

fn parse_jin10(body: &str) -> FeedResult {
    let json = body
        .trim()
        .strip_prefix("var newest")
        .and_then(|body| body.trim_start().strip_prefix('='))
        .ok_or(DataError::Missing("flash_list"))?
        .trim()
        .trim_end_matches(';');
    let items: Vec<Jin10Item> = serde_json::from_str(json)?;
    Ok(items
        .into_iter()
        .filter_map(|item| {
            if item.channel.is_some_and(|channels| channels.contains(&5)) {
                return None;
            }
            let title = item
                .data
                .title
                .filter(|title| !title.trim().is_empty())
                .or(item.data.content)?;
            let title = if title.contains(['<', '&']) {
                scraper::Html::parse_fragment(&title)
                    .root_element()
                    .text()
                    .collect::<String>()
            } else {
                title
            };
            let title = title.trim();
            let headline = title
                .strip_prefix('【')
                .and_then(|text| text.split_once('】'))
                .map(|(headline, _)| headline)
                .filter(|headline| !headline.trim().is_empty())
                .unwrap_or(title);
            let url = format!("https://flash.jin10.com/detail/{}", item.id);
            Story::parse(Some(headline.to_owned()), Some(url))
        })
        .take(LIMIT)
        .collect())
}

pub fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_boundary_rejects_empty_titles_and_non_web_links() {
        assert!(Story::parse(Some("\n\0".into()), Some("https://example.org".into())).is_none());
        for url in [
            "file:///secret",
            "javascript:alert(1)",
            "https://",
            "relative",
        ] {
            assert!(Story::parse(Some("title".into()), Some(url.into())).is_none());
        }
        let story = Story::parse(
            Some("  title\nline  ".into()),
            Some("https://example.org".into()),
        )
        .unwrap();
        assert_eq!(
            String::from_utf16_lossy(&story.title[..story.title.len() - 1]),
            "title line"
        );
    }

    #[test]
    fn zhihu_uses_questions_not_search_words() {
        let body = r#"{"data":[{"target":{"title_area":{"text":"question"},"link":{"url":"https://www.zhihu.com/question/123"}}},{"target":{"title_area":{"text":"search"},"link":{"url":"https://www.zhihu.com/search?q=word"}}}]}"#;
        let stories = parse_zhihu(body).unwrap();
        assert_eq!(stories.len(), 1);
        assert!(parse_zhihu(r#"{"unrelated":[]}"#).is_err());
    }

    #[test]
    fn weibo_requires_success_and_confirmed_badge_fields() {
        assert!(parse_weibo(r#"{"ok":0}"#).is_err());
        assert!(parse_weibo(r#"{"ok":1}"#).is_err());
        let stories = parse_weibo(r#"{"ok":1,"data":{"realtime":[{"word":"first","label_name":"热"},{"word":"second","label_name":"unknown"}]}}"#).unwrap();
        assert_eq!(stories.len(), 2);
        assert_eq!(stories[0].badge, Some(Badge::Hot));
        assert_eq!(stories[1].badge, None);
    }
}
