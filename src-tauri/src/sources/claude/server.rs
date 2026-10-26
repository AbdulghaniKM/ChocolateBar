use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use tokio::net::TcpListener;

use super::service::HookService;

pub const TOKEN_HEADER: &str = "x-chocolatebar-token";
const STOP_POLL: Duration = Duration::from_millis(200);
const BIND_RETRY: Duration = Duration::from_secs(5);

#[derive(Clone)]
struct ServerState {
    service: HookService,
    token: Arc<str>,
}

pub struct RunningServer {
    stop: Arc<AtomicBool>,
    listening: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl RunningServer {
    pub fn start(service: HookService, port: u16, token: &str) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let listening = Arc::new(AtomicBool::new(false));
        let state = ServerState { service, token: Arc::from(token) };
        let thread = spawn_server_thread(state, port, Arc::clone(&stop), Arc::clone(&listening));
        Self { stop, listening, thread }
    }

    pub fn is_listening(&self) -> bool {
        self.listening.load(Ordering::Relaxed)
    }

    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                log::warn!("claude hook server thread ended with a panic");
            }
        }
    }
}

fn spawn_server_thread(
    state: ServerState,
    port: u16,
    stop: Arc<AtomicBool>,
    listening: Arc<AtomicBool>,
) -> Option<JoinHandle<()>> {
    let spawned = thread::Builder::new()
        .name("claude-hook-server".into())
        .spawn(move || run_runtime(state, port, stop, listening));
    spawned.map_err(|error| log::error!("claude hook server failed to spawn: {error}")).ok()
}

fn run_runtime(state: ServerState, port: u16, stop: Arc<AtomicBool>, listening: Arc<AtomicBool>) {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build();
    match runtime {
        Ok(runtime) => runtime.block_on(serve_until_stopped(state, port, stop, listening)),
        Err(error) => log::error!("claude hook server runtime failed: {error}"),
    }
}

async fn serve_until_stopped(
    state: ServerState,
    port: u16,
    stop: Arc<AtomicBool>,
    listening: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        match TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await {
            Ok(listener) => {
                listening.store(true, Ordering::Relaxed);
                serve(listener, state.clone(), &stop).await;
                listening.store(false, Ordering::Relaxed);
                return;
            }
            Err(error) => {
                log::warn!("claude hook server cannot bind 127.0.0.1:{port}: {error}");
                sleep_unless_stopped(&stop, BIND_RETRY).await;
            }
        }
    }
}

async fn serve(listener: TcpListener, state: ServerState, stop: &AtomicBool) {
    let app = Router::new().route("/hook/{event}", post(receive_hook)).with_state(state);
    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(error) = result {
                log::error!("claude hook server stopped: {error}");
            }
        }
        () = wait_for_stop(stop) => {}
    }
}

async fn wait_for_stop(stop: &AtomicBool) {
    while !stop.load(Ordering::Relaxed) {
        tokio::time::sleep(STOP_POLL).await;
    }
}

async fn sleep_unless_stopped(stop: &AtomicBool, duration: Duration) {
    tokio::select! {
        () = tokio::time::sleep(duration) => {}
        () = wait_for_stop(stop) => {}
    }
}

async fn receive_hook(
    State(state): State<ServerState>,
    Path(event): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    if !carries_valid_token(&headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(state.service.handle(&event, &body).await))
}

fn carries_valid_token(headers: &HeaderMap, expected: &str) -> bool {
    let supplied = headers.get(TOKEN_HEADER).and_then(|value| value.to_str().ok());
    supplied.is_some_and(|supplied| tokens_match(supplied, expected))
}

fn tokens_match(supplied: &str, expected: &str) -> bool {
    if supplied.len() != expected.len() || expected.is_empty() {
        return false;
    }
    let differing_bits = supplied
        .bytes()
        .zip(expected.bytes())
        .fold(0u8, |accumulated, (left, right)| accumulated | (left ^ right));
    differing_bits == 0
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    #[test]
    fn matching_tokens_are_accepted() {
        assert!(tokens_match("abc123", "abc123"));
    }

    #[test]
    fn wrong_or_empty_tokens_are_rejected() {
        assert!(!tokens_match("abc124", "abc123"));
        assert!(!tokens_match("abc", "abc123"));
        assert!(!tokens_match("", ""));
    }

    #[test]
    fn requests_without_the_header_are_rejected() {
        assert!(!carries_valid_token(&HeaderMap::new(), "secret"));
    }

    #[test]
    fn requests_with_the_header_are_accepted() {
        let mut headers = HeaderMap::new();
        headers.insert(TOKEN_HEADER, HeaderValue::from_static("secret"));
        assert!(carries_valid_token(&headers, "secret"));
    }
}
