use crate::{mutex_lock, ApiExtension};
use axum::response::IntoResponse;
use axum::{debug_handler, Router};
use log::info;
use once_cell::sync::Lazy;
use std::fmt;
use std::sync::Mutex;

static COLLECTED_ROUTES: Lazy<Mutex<Vec<&'static str>>> =
    Lazy::new(|| Mutex::new(Default::default()));

macro add_route($router:expr, $t:tt $path:literal, $f:expr) {
    paste::paste! {
        $router = $router.route($path, ::axum::routing::[<$t:lower>]($f));
        mutex_lock!(COLLECTED_ROUTES).push(concat!(stringify!([<$t:upper>]), " ", $path));
    }
}

pub fn router() -> Router {
    let mut router = Router::new();
    add_route!(router, GET "/test", test_api);
    add_route!(router, GET "/routes", list_routes);
    router
}

pub async fn list_routes() -> impl IntoResponse {
    info!("Route: /routes");
    let mut content = String::new();
    use fmt::Write;
    for &line in &*mutex_lock!(COLLECTED_ROUTES) {
        writeln!(&mut content, "{}", line).unwrap();
    }
    content
}

pub macro api_error {
() => {
        crate::ResponseJson::<()>::error().into_response()
    },
($message:expr) => {{
        log::debug!("Error message:\n{}", $message);
        crate::ResponseJson::<()>::error_msg($message).into_response()
    }}
}

pub macro handle_errors($r:expr) {{
    log::debug!("Result: {:?}", &$r);
    let err = $r.err().unwrap();
    api_error!(format!("{}", err))
}}

#[debug_handler]
pub async fn test_api(ext: ApiExtension) -> impl IntoResponse {
    let _db = &ext.db;
    "hello, world"
}
