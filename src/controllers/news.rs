use actix_web::{get, web, HttpResponse};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use reqwest::get;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env::var;
use rand::thread_rng;
use rand::seq::SliceRandom;
use crate::config::helpers::ResponseJson;

#[derive(Debug, Deserialize)]
struct QueryParams {
    search: Option<String>,
}

macro_rules! init_env_utils {
    ($gnews_url: expr, $api_key: expr) => {{
        $gnews_url = var("GNEWS_URL").unwrap();
        $api_key = var("GNEWS_API_KEY").unwrap();
    }};
}
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct News {
    pub author: Option<Value>,
    pub description: Option<Value>,
    pub content: Value,
    pub url: Value,
    pub publishedAt: Value,
    pub image: Value,
}

async fn get_news_api<'a>(url: &String, mut conn: MultiplexedConnection) -> Value {
    let news = get(url).await.unwrap().text().await.unwrap();
    let mut news_json: Value = serde_json::from_str(&news).unwrap();
    let articles = news_json["articles"].as_array_mut().unwrap();
    let mut rng = thread_rng();
    articles.shuffle(&mut rng);
    let articles = articles
        .into_iter()
        .take(5)
        .map(|article| {
            let mut article = article.clone();
            article["image"] = Value::Null;
            article["url"] = Value::Null;
            article
        })
        .collect();
    news_json["articles"] = Value::Array(articles);
    let news_json = news_json;
    
    let _ = conn
        .set_ex::<&str, &String, String>("news", &news_json.to_string(), 10)
        .await;
    news_json
}

fn sanitize_data(news_args: &Value) -> Vec<News> {
    let mut binding: Vec<News> = vec![];
    if let Some(news) = news_args["articles"].as_array() {
        for _news in news {
            binding.push(News {
                author: Some(_news["source"]["name"].clone()),
                content: _news["content"].clone(),
                description: Some(_news["description"].clone()),
                url: _news["url"].clone(),
                image: _news["image"].clone(),
                publishedAt: _news["publishedAt"].clone(),
            })
        }
    };
    binding
}
// pub fn get_top_headlines() -> serde_json::Value {}
pub async fn get_news(search: &Option<String>) -> Result<Vec<News>, String> {
    let stage = var("STAGE").unwrap();
    let redis_url = if stage == "DEVELOPMENT" {
        var("REDIS_DEV_URL").unwrap()
    } else {
        var("REDIS_URL").unwrap()
    };
    // let redis_url = ;
    let conn = match redis::Client::open(redis_url) {
        Ok(conn) => conn,
        Err(err) => {
            println!("{:?}", err);
            return serde_json::from_str("[]").unwrap();
        }
    };
    let mut connection = conn.get_multiplexed_async_connection().await.unwrap();

    // println!("NAMA {:?}", name);
    let cached_news = match connection.get::<&str, Option<String>>("news").await {
        Ok(news) => news,
        Err(err) => {
            println!("{:?}", err);
            None
        }
    };
    let gnews_url;
    let api_key;
    init_env_utils!(gnews_url, api_key);
    // match condition to filter search or not
    let news = match search {
        None => match cached_news {
            Some(news) => sanitize_data(&serde_json::from_str(&news).unwrap()),
            None => {
                let gnews_url = format!("{gnews_url}/top-headlines?lang=en&apikey={api_key}");
                let news = sanitize_data(&get_news_api(&gnews_url, connection).await);
                
                news
            }
        },
        Some(search) => {
            let gnews_url = format!(r#"{gnews_url}/search?q="{search}"&lang=en&apikey={api_key}"#);
            sanitize_data(&get_news_api(&gnews_url, connection).await)
        }
    };
    Ok(news)
}

#[get("/news")]
pub async fn index(q: web::Query<QueryParams>) -> HttpResponse {
    let news = match get_news(&q.search).await {
        Ok(news) => news,
        Err(err) => {
            println!("{:?}", err);
            return HttpResponse::BadRequest().json(ResponseJson::<Vec<News>> {
                data: None,
                message: err,
                status: crate::config::helpers::Status::FAIL,
                status_code: 400,
            });
        }
    };

    HttpResponse::Ok().json(ResponseJson::<Vec<News>> {
        data: Some(news),
        message: String::from("Success to retrive news"),
        status: crate::config::helpers::Status::SUCCESS,
        status_code: 200,
    })
}
