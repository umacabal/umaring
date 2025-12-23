use crate::ring::{HealthStatus, Ring};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

const RING_JS_PATTERN: &str = "umaring.mkr.cx/ring.js";
const API_PATTERN: &str = "umaring.mkr.cx/";
const REDIRECT_PREV: &str = "/prev";
const REDIRECT_NEXT: &str = "/next";

pub async fn check_site(url: &str) -> HealthStatus {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .unwrap();

    // Fetch the index page
    let html = match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(text) => text,
            Err(_) => return HealthStatus::UnhealthyDown,
        },
        Ok(_) => return HealthStatus::UnhealthyDown,
        Err(_) => return HealthStatus::UnhealthyDown,
    };

    let html_lower = html.to_lowercase();

    // Priority 1: Check for ring.js script inclusion
    if html_lower.contains(RING_JS_PATTERN) {
        return HealthStatus::HealthyRingJs;
    }

    // Priority 2: Check for API URL pattern (umaring.mkr.cx/username)
    // Need to check if it's a redirect link pattern or API fetch
    if html_lower.contains(API_PATTERN) {
        if has_redirect_pattern(&html_lower) {
            return HealthStatus::HealthyRedirectLinks;
        }
        return HealthStatus::HealthyApiJs;
    }

    // Priority 3: Check linked JS files
    let base_url = extract_base_url(url);
    let js_urls = extract_js_urls(&html, url, &base_url);

    for js_url in js_urls {
        match client.get(&js_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(js_content) = resp.text().await {
                    let js_lower = js_content.to_lowercase();

                    // Check JS file for ring.js reference
                    if js_lower.contains(RING_JS_PATTERN) {
                        return HealthStatus::HealthyRingJs;
                    }

                    // Check JS file for API pattern
                    if js_lower.contains(API_PATTERN) {
                        if has_redirect_pattern(&js_lower) {
                            return HealthStatus::HealthyRedirectLinks;
                        }
                        return HealthStatus::HealthyApiJs;
                    }

                    // Generic umaring reference in JS
                    if js_lower.contains("umaring") {
                        return HealthStatus::HealthyJsOther;
                    }
                }
            }
            _ => continue,
        }
    }

    // Priority 4: Check for generic "umaring" in HTML (server-side/static integration)
    if html_lower.contains("umaring") {
        return HealthStatus::HealthyStatic;
    }

    HealthStatus::UnhealthyMissing
}

/// Check if content contains redirect link patterns like /prev or /next after umaring.mkr.cx/
fn has_redirect_pattern(content: &str) -> bool {
    // Look for patterns like umaring.mkr.cx/username/prev or umaring.mkr.cx/username/next
    if let Some(pos) = content.find(API_PATTERN) {
        let after_api = &content[pos + API_PATTERN.len()..];
        // Check if somewhere after the API pattern we have /prev or /next
        return after_api.contains(REDIRECT_PREV) || after_api.contains(REDIRECT_NEXT);
    }
    false
}

fn extract_base_url(url: &str) -> String {
    // Extract protocol and domain (e.g., "https://example.com")
    if let Some(idx) = url.find("://") {
        let after_protocol = &url[idx + 3..];
        if let Some(slash_idx) = after_protocol.find('/') {
            return url[..idx + 3 + slash_idx].to_string();
        }
    }
    url.trim_end_matches('/').to_string()
}

fn extract_js_urls(html: &str, page_url: &str, base_url: &str) -> Vec<String> {
    let mut js_urls = Vec::new();

    // Simple regex-like extraction for script src attributes
    // Matches src="..." or src='...'
    let patterns = [
        (r#"<script"#, r#"src=""#, r#"""#),
        (r#"<script"#, r#"src='"#, r#"'"#),
    ];

    for (tag_start, attr_start, attr_end) in patterns {
        let mut search_pos = 0;
        while let Some(tag_pos) = html[search_pos..].find(tag_start) {
            let tag_pos = search_pos + tag_pos;
            // Find the end of this tag
            let tag_end = html[tag_pos..].find('>').map(|p| tag_pos + p).unwrap_or(html.len());
            let tag_content = &html[tag_pos..tag_end];

            if let Some(src_pos) = tag_content.find(attr_start) {
                let src_start = src_pos + attr_start.len();
                if let Some(src_end) = tag_content[src_start..].find(attr_end) {
                    let src = &tag_content[src_start..src_start + src_end];
                    let resolved = resolve_url(src, page_url, base_url);
                    if !js_urls.contains(&resolved) {
                        js_urls.push(resolved);
                    }
                }
            }
            search_pos = tag_pos + 1;
        }
    }

    js_urls
}

fn resolve_url(src: &str, page_url: &str, base_url: &str) -> String {
    if src.starts_with("http://") || src.starts_with("https://") {
        // Absolute URL
        src.to_string()
    } else if src.starts_with("//") {
        // Protocol-relative URL
        format!("https:{}", src)
    } else if src.starts_with('/') {
        // Root-relative URL
        format!("{}{}", base_url, src)
    } else {
        // Relative URL
        let page_base = page_url.trim_end_matches('/');
        format!("{}/{}", page_base, src)
    }
}

/// Scan all members on startup
pub async fn scan_all(ring: Arc<RwLock<Ring>>) {
    let members: Vec<(String, String)> = {
        let ring = ring.read().await;
        ring.all_with_health()
            .into_iter()
            .map(|(m, _)| (m.id.clone(), m.url.clone()))
            .collect()
    };

    for (id, url) in members {
        let status = check_site(&url).await;
        let mut ring = ring.write().await;
        ring.set_health(&id, status);
    }
}

/// Background task that checks one member per minute
pub async fn health_check_loop(ring: Arc<RwLock<Ring>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;

        let (id, url) = {
            let mut ring = ring.write().await;
            let member = ring.next_member_to_check();
            (member.id.clone(), member.url.clone())
        };

        let status = check_site(&url).await;
        let mut ring = ring.write().await;
        ring.set_health(&id, status);
    }
}
