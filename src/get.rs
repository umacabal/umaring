use axum::{
    extract::{Path, Query, State},
    response::Response,
    http::header::CONTENT_TYPE,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{sync::RwLock, fs::read_to_string};

use crate::ring::{Member, Ring};

static RING_JS: &str = include_str!("../js/ring.js");

#[derive(Deserialize)]
pub struct ScriptParams {
    id: String,
    mode: Option<String>
}

#[derive(Serialize)]
struct MemberGetResponse {
    prev: Member,
    member: Member,
    next: Member,
}

async fn get_member_response(ring: Arc<RwLock<Ring>>, id: String) -> Option<MemberGetResponse> {
    let ring = ring.read().await;
    let member = ring.get(&id);

    if member.is_none() {
        return None;
    }

    let member = member.unwrap();

    let (prev, next) = ring.neighbors(&id).unwrap();

    Some(MemberGetResponse {
        member: member.clone(),
        next: next.clone(),
        prev: prev.clone(),
    })
}

pub async fn landing_page() -> Result<Response<String>, std::convert::Infallible> {
    match read_to_string("index.html").await {
        Ok(content) => Ok(Response::builder()
            .header(CONTENT_TYPE, "text/html")
            .body(content)
            .unwrap()),
        Err(_) => Ok(Response::builder()
            .status(404)
            .body("HTML file not found".to_string())
            .unwrap()),
    }
}

pub async fn serve_css() -> Result<Response<String>, std::convert::Infallible> {
    match read_to_string("styles.css").await {
        Ok(content) => Ok(Response::builder()
            .header(CONTENT_TYPE, "text/css")
            .body(content)
            .unwrap()),
        Err(_) => Ok(Response::builder()
            .status(404)
            .body("CSS not found".to_string())
            .unwrap()),
    }
}

pub async fn one(
    State(ring): State<Arc<RwLock<Ring>>>,
    Path(id): Path<String>,
) -> Result<Response<String>, std::convert::Infallible> {
    let response = get_member_response(ring, id).await;
    if response.is_none() {
        return member_not_found();
    }

    json_response(response.unwrap())
}

// Send temporary redirect to the prev member
pub async fn prev(
    State(ring): State<Arc<RwLock<Ring>>>,
    Path(id): Path<String>,
) -> Result<Response<String>, std::convert::Infallible> {
    let ring = ring.read().await;
    let member = ring.get(&id);

    if member.is_none() {
        return member_not_found();
    }

    let (prev, _) = ring.neighbors(&id).unwrap();

    temporary_redirect(&prev.url)
}

pub async fn next(
    State(ring): State<Arc<RwLock<Ring>>>,
    Path(id): Path<String>,
) -> Result<Response<String>, std::convert::Infallible> {
    let ring = ring.read().await;
    let member = ring.get(&id);

    if member.is_none() {
        return member_not_found();
    }

    let (_, next) = ring.neighbors(&id).unwrap();

    temporary_redirect(&next.url)
}

pub async fn all(
    State(ring): State<Arc<RwLock<Ring>>>,
) -> Result<Response<String>, std::convert::Infallible> {
    let ring = ring.read().await;

    let members = ring.iter().collect::<Vec<&Member>>();
    json_response(members)
}

pub async fn ring_js(
    State(ring): State<Arc<RwLock<Ring>>>,
    params: Query<ScriptParams>,
) -> Result<Response<String>, std::convert::Infallible> {
    let response = get_member_response(ring, params.id.clone()).await;
    if response.is_none() {
        return member_not_found();
    }

    let user_js = RING_JS
        .replace("JSON_DATA_HERE", &serde_json::to_string(&response.unwrap()).unwrap())
        .replace("MODE_PARAM_HERE", &params.mode.clone().unwrap_or("base".to_string()));

    Ok(Response::builder()
        .header("Content-Type", "text/javascript")
        .body(user_js)
        .unwrap())
}

fn temporary_redirect(url: &str) -> Result<Response<String>, std::convert::Infallible> {
    Ok(Response::builder()
        .status(302)
        .header("Location", url)
        .body("".to_string())
        .unwrap())
}

fn json_response<T: Serialize>(data: T) -> Result<Response<String>, std::convert::Infallible> {
    let json = serde_json::to_string(&data).unwrap();
    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(json)
        .unwrap())
}

fn member_not_found() -> Result<Response<String>, std::convert::Infallible> {
    Ok(Response::builder()
        .status(404)
        .body("Member not found".to_string())
        .unwrap())
}
