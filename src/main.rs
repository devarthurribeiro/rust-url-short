use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    let db = models::blank_db();

    let site = warp::get()
        .and(warp::path::end())
        .and(warp::fs::dir("./src/html/index.html"));

    let not_found = warp::path("not_found")
        .and(warp::path::end())
        .and(warp::fs::file("./src/html/not_found.html"));

    let api = filters::urls(db).or(not_found).or(site);

    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}

mod filters {
    use super::handlers;
    use super::models::{Db, Url};
    use warp::Filter;

    pub fn urls(
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        url_create(db.clone()).or(url_get(db.clone()))
    }

    pub fn url_get(
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("urls" / String)
            .and(warp::get())
            .and(with_db(db))
            .and_then(handlers::get_url)
    }

    pub fn url_create(
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("urls")
            .and(warp::post())
            .and(json_body())
            .and(with_db(db))
            .and_then(handlers::create_url)
    }

    fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
    fn json_body() -> impl Filter<Extract = (Url,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}

mod handlers {
    use super::models::{Db, Url};
    use std::convert::Infallible;
    use warp::{http::StatusCode, http::Uri};

    pub async fn get_url(slug: String, db: Db) -> Result<impl warp::Reply, Infallible> {
        let urls = db.lock().await;
        for url in urls.iter() {
            if url.slug == slug {
                let location = url.url.parse::<Uri>().unwrap();
                return Ok(warp::redirect(location));
            }
        }
        Ok(warp::redirect(Uri::from_static("/not_found")))
    }

    pub async fn create_url(url: Url, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut vec = db.lock().await;
        vec.push(url);
        Ok(StatusCode::CREATED)
    }
}

mod models {
    use serde_derive::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub type Db = Arc<Mutex<Vec<Url>>>;

    pub fn blank_db() -> Db {
        Arc::new(Mutex::new(Vec::new()))
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Url {
        pub url: String,
        pub slug: String,
    }
}
