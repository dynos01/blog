use axum::{extract::Path, http::StatusCode, response::IntoResponse};
use hashbrown::HashMap;
use log::debug;
use once_cell::sync::Lazy;

static ASSETS: Lazy<HashMap<&'static str, (&'static [u8], &'static str)>> = Lazy::new(|| {
    mod asset {
        include!(concat!(env!("OUT_DIR"), "/asset.rs"));
    }

    asset::ASSETS
        .into_iter()
        .map(|(path, data, mime_type)| {
            debug!("Loading asset {path}");
            (*path, (*data, *mime_type))
        })
        .collect()
});

pub async fn asset_handler(Path(url): Path<String>) -> impl IntoResponse {
    let (data, mime_type) = match ASSETS.get(&*url) {
        Some((data, mime_type)) => (*data, *mime_type),
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let headers = [("Content-Type", mime_type)];

    (StatusCode::OK, headers, data).into_response()
}
