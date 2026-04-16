use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
    response::Response,
    routing::any,
    Router,
};
use reqwest::Client;

use crate::config::project::ProxyRoute;
use crate::config::resolver::{self, SecretStores};

/// Shared state available to every axum handler.
#[derive(Clone)]
struct ProxyState {
    routes: Arc<Vec<ResolvedRoute>>,
    client: Client,
}

/// A route where all header templates have been resolved against the secrets store.
#[derive(Clone)]
struct ResolvedRoute {
    path_prefix: String,
    target: String,
    headers: HashMap<String, String>,
}

/// Resolve all secret references in route headers up-front, at startup.
pub fn build_router(routes: Vec<ProxyRoute>, stores: &SecretStores) -> Result<Router> {
    let mut resolved = Vec::with_capacity(routes.len());
    for route in routes {
        let mut headers = HashMap::new();
        for (k, v) in &route.inject_headers {
            let value = resolver::expand_template(v, stores)?;
            headers.insert(k.clone(), value);
        }
        resolved.push(ResolvedRoute {
            path_prefix: route.path.clone(),
            target: route.target.trim_end_matches('/').to_string(),
            headers,
        });
    }

    let state = ProxyState {
        routes: Arc::new(resolved),
        client: Client::new(),
    };

    Ok(Router::new()
        .route("/{*path}", any(proxy_handler))
        .with_state(state))
}

/// Forward an incoming request to the matching upstream target.
async fn proxy_handler(
    State(state): State<ProxyState>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let req_path = req.uri().path().to_string();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    // Find the first matching route prefix
    let route = state
        .routes
        .iter()
        .find(|r| req_path.starts_with(&r.path_prefix))
        .ok_or(StatusCode::NOT_FOUND)?;

    // Strip the route prefix and build the upstream URL
    let remainder = &req_path[route.path_prefix.len()..];
    let upstream_url = format!("{}{}{}", route.target, remainder, query);

    // Build the upstream request
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut upstream_req = state.client.request(method, &upstream_url);

    // Forward original headers (excluding host)
    let mut fwd_headers = reqwest::header::HeaderMap::new();
    for (name, value) in req.headers() {
        if name == axum::http::header::HOST {
            continue;
        }
        if let (Ok(n), Ok(v)) = (
            reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()),
            reqwest::header::HeaderValue::from_bytes(value.as_bytes()),
        ) {
            fwd_headers.insert(n, v);
        }
    }

    // Inject route-specific headers (overwrite if already present)
    for (k, v) in &route.headers {
        if let (Ok(n), Ok(val)) = (
            reqwest::header::HeaderName::from_bytes(k.as_bytes()),
            reqwest::header::HeaderValue::from_str(v),
        ) {
            fwd_headers.insert(n, val);
        }
    }
    upstream_req = upstream_req.headers(fwd_headers);

    // Forward body
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    upstream_req = upstream_req.body(body_bytes);

    // Execute
    let upstream_resp = upstream_req
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    // Convert response back to axum
    let status = axum::http::StatusCode::from_u16(upstream_resp.status().as_u16())
        .unwrap_or(axum::http::StatusCode::BAD_GATEWAY);
    let mut resp_headers = HeaderMap::new();
    for (k, v) in upstream_resp.headers() {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(k.as_str().as_bytes()),
            HeaderValue::from_bytes(v.as_bytes()),
        ) {
            resp_headers.insert(name, value);
        }
    }

    let body_bytes = upstream_resp
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let mut response = Response::new(Body::from(body_bytes));
    *response.status_mut() = status;
    *response.headers_mut() = resp_headers;
    Ok(response)
}
