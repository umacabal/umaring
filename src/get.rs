use axum::{
    extract::{Path, Query, State},
    response::Response
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{sync::RwLock};

use crate::ring::{HealthStatus, Member, Ring};

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

#[derive(Serialize)]
struct MemberStatusResponse {
    id: String,
    name: String,
    url: String,
    status: HealthStatus,
    status_description: &'static str,
    healthy: bool,
    last_checked: Option<u64>,
    last_checked_ago: Option<String>,
}

#[derive(Serialize, Default)]
struct ByStatus {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    unknown: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ring_js: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    api_js: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    redirect_links: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    static_html: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    js_other: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    down: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    missing: Vec<String>,
}

#[derive(Serialize)]
struct StatusResponse {
    total: usize,
    healthy: usize,
    unhealthy: usize,
    by_status: ByStatus,
    members: Vec<MemberStatusResponse>,
}

pub async fn status(
    State(ring): State<Arc<RwLock<Ring>>>,
) -> Result<Response<String>, std::convert::Infallible> {
    let ring = ring.read().await;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::new(0, 0))
        .as_secs();

    let members: Vec<MemberStatusResponse> = ring
        .all_with_health()
        .into_iter()
        .map(|(member, health)| {
            let last_checked_ago = health.last_checked.map(|ts| {
                let secs = now.saturating_sub(ts);
                if secs < 60 {
                    format!("{}s ago", secs)
                } else if secs < 3600 {
                    format!("{}m ago", secs / 60)
                } else {
                    format!("{}h ago", secs / 3600)
                }
            });

            MemberStatusResponse {
                id: member.id.clone(),
                name: member.name.clone(),
                url: member.url.clone(),
                status: health.status.clone(),
                status_description: health.status.description(),
                healthy: health.status.is_healthy(),
                last_checked: health.last_checked,
                last_checked_ago,
            }
        })
        .collect();

    let healthy_count = members.iter().filter(|m| m.healthy).count();

    let mut by_status = ByStatus::default();
    for m in &members {
        match m.status {
            HealthStatus::Unknown => by_status.unknown.push(m.id.clone()),
            HealthStatus::HealthyRingJs => by_status.ring_js.push(m.id.clone()),
            HealthStatus::HealthyApiJs => by_status.api_js.push(m.id.clone()),
            HealthStatus::HealthyRedirectLinks => by_status.redirect_links.push(m.id.clone()),
            HealthStatus::HealthyStatic => by_status.static_html.push(m.id.clone()),
            HealthStatus::HealthyJsOther => by_status.js_other.push(m.id.clone()),
            HealthStatus::UnhealthyDown => by_status.down.push(m.id.clone()),
            HealthStatus::UnhealthyMissing => by_status.missing.push(m.id.clone()),
        }
    }

    let response = StatusResponse {
        total: members.len(),
        healthy: healthy_count,
        unhealthy: members.len() - healthy_count,
        by_status,
        members,
    };

    json_response(response)
}
