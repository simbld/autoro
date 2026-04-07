// /backend/src/news.rs
// Client NewsAPI.org pour récupérer les actualités d'un instrument

use reqwest::Client;
use serde::Deserialize;

use crate::models::{NewsArticle, NewsResponse};

// ── Structs internes (désérialisation NewsAPI) ────────────────────────────────

#[derive(Debug, Deserialize)]
struct NewsApiSource {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewsApiArticle {
    source: NewsApiSource,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    url_to_image: Option<String>,
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsApiResponse {
    articles: Vec<NewsApiArticle>,
}

// ── Mapping symbole → terme de recherche ─────────────────────────────────────

fn symbol_to_query(symbol: &str) -> &str {
    match symbol.to_uppercase().as_str() {
        "BTC" => "Bitcoin",
        "ETH" => "Ethereum",
        "SOL" => "Solana",
        "XRP" => "Ripple XRP",
        "BNB" => "Binance BNB",
        _ => symbol,
    }
}

// ── Client NewsAPI ────────────────────────────────────────────────────────────

pub async fn fetch_news(
    api_key: &str,
    symbol: &str,
) -> Result<NewsResponse, reqwest::Error> {
    let query = symbol_to_query(symbol);
    let client = Client::new();
    let url = format!(
        "https://newsapi.org/v2/everything\
         ?q={}\
         &language=en\
         &sortBy=publishedAt\
         &pageSize=20\
         &apiKey={}",
        query, api_key
    );

    let resp: NewsApiResponse = client.get(&url).send().await?.json().await?;

    let articles: Vec<NewsArticle> = resp
        .articles
        .into_iter()
        .filter(|a| a.title.as_deref().unwrap_or("") != "[Removed]")
        .map(|a| NewsArticle {
            title: a.title.unwrap_or_default(),
            description: a.description,
            url: a.url.unwrap_or_default(),
            image_url: a.url_to_image,
            published_at: a.published_at.unwrap_or_default(),
            source_name: a.source.name.unwrap_or_default(),
        })
        .collect();

    Ok(NewsResponse { articles })
}