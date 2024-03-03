use axum::http::{HeaderMap, Request, Response};
use axum::middleware::Next;
use axum::body::{Body};
use crate::errors::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    CdEvent,
    CloudEvent,
}

pub async fn event_filter(
    headers: HeaderMap,
    mut request: Request<Body>,
    next: Next,
)  -> Result<Response<Body>, Error> {
    let has_header_event = headers.iter().filter(|(key, _)| key.to_string().starts_with("ce-")
        || key.to_string().starts_with("ce_")).count() > 0;
    if has_header_event {
        request.extensions_mut().insert(EventType::CloudEvent);
    } else {
        request.extensions_mut().insert(EventType::CdEvent);
    }
    Ok(next.run(request).await)
}
