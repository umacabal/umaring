use axum::{
    extract::{Path, State},
    response::Response,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ring::{Member, Ring};

#[derive(Serialize)]
struct MemberGetResponse {
    prev: Member,
    member: Member,
    next: Member,
}

pub async fn one(
    State(ring): State<Arc<RwLock<Ring>>>,
    Path(id): Path<String>,
) -> Result<Response<String>, std::convert::Infallible> {
    let ring = ring.read().await;
    let member = ring.get(&id);

    if member.is_none() {
        return member_not_found();
    }

    let member = member.unwrap();

    let (prev, next) = ring.neighbors(&id).unwrap();

    let response = MemberGetResponse {
        member: member.clone(),
        next: next.clone(),
        prev: prev.clone(),
    };

    json_response(response)
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
