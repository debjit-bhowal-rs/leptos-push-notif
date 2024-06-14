use axum::{extract::{Path, Request, State}, response::{IntoResponse, Response}, routing::get};
use leptos::provide_context;
use leptos_axum::handle_server_fns_with_context;
use leptos_push_notif::{app::App, state::server::AppState};

async fn server_fn_handler(
    State(app_state): State<AppState>,
    path: Path<String>,
    request: Request<axum::body::Body>,
) -> impl IntoResponse {
    log::info!("{:?}", path);

    handle_server_fns_with_context(
        move || {
            provide_context(app_state.push_kv.clone());
            provide_context(app_state.push_client.clone());
            provide_context(app_state.sig_b.clone());
        },
        request,
    )
    .await
}

async fn leptos_routes_handler(
    State(app_state): State<AppState>,
    req: Request<axum::body::Body>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            provide_context(app_state.push_kv.clone());
            provide_context(app_state.push_client.clone());
            provide_context(app_state.sig_b.clone());
        },
        App,
    );
    handler(req).await.into_response()
}



#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_push_notif::app::*;
    use leptos_push_notif::fileserv::file_and_error_handler;

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    let app_state = AppState::new(leptos_options, routes.clone()).unwrap();

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
