use rocket::{get, response::content::Html};

const INDEX_HTML: &str = include_str!("../index.html");

#[get("/")]
pub async fn root() -> Html<&'static str> {
    Html(INDEX_HTML)
}

#[get("/<_page>", rank = 99)]
pub async fn wildcard(_page: String) -> Html<&'static str> {
    Html(INDEX_HTML)
}
