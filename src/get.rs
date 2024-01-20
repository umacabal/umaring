
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    extract::{State, Path},
    response::Response,
};

use crate::ring::{Ring, Member};

#[derive(Serialize)]
struct MemberGetResponse {
    prev: Option<Member>,
    member: Member,
    next: Option<Member>,
}

pub async fn one(
    State(state): State<Arc<RwLock<Ring>>>,
    Path(id): Path<String>,
) -> Result<Response<String>, std::convert::Infallible> {
    let state = state.read().await;
    let member_link = state.members.get(&id);

    if member_link.is_none() {
        return Ok(
            Response::builder()
                .status(404)
                .body("Member not found".to_string())
                .unwrap()
        );
    }

    let member_link = member_link.unwrap();

    let member = member_link.read().await.member.clone();

    let next = match &member_link.read().await.next {
        Some(next) => Some(next.read().await.member.clone()),
        None => None,
    };

    let prev = match &member_link.read().await.prev {
        Some(prev) => Some(prev.read().await.member.clone()),
        None => None,
    };

    let response = MemberGetResponse {
        member,
        next,
        prev,
    };

    let response_json = serde_json::to_string(&response).unwrap();
    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(response_json)
        .unwrap()
    )
}

pub async fn all(
    State(state): State<Arc<RwLock<Ring>>>,
) -> Result<Response<String>, std::convert::Infallible> {
    let state = state.read().await;
    let members = state.all.clone();
    let members_json = serde_json::to_string(&members).unwrap();
    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(members_json)
        .unwrap()
    )
}