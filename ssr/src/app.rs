use crate::{error_template::{AppError, ErrorTemplate}, push::broadcast_message, state::push_worker::PushWorkerCtx};

use leptos::*;
use leptos_meta::*;
use leptos_router::*;


#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let push_ctx = PushWorkerCtx::default();
    provide_context(push_ctx);

    #[cfg(feature = "hydrate")]
    create_local_resource(move || (), move |_| async move {
        use wasm_bindgen_futures::JsFuture;
        use web_sys::ServiceWorkerRegistration;

        log::info!("registring worker");
        let nav = window().navigator();
        let sw = nav.service_worker();
        let sw_prom = sw.register("/leptos-push-notif-worker.js");
        let sw_reg_js = JsFuture::from(sw_prom).await.unwrap();
        let sw_reg = ServiceWorkerRegistration::from(sw_reg_js);
        log::info!("registered");
        push_ctx.worker.set(Some(sw_reg));
        push_ctx.ready.set(true);
    });

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos-push-notif.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[cfg(feature = "hydrate")]
async fn perform_subscription(perm_prom: js_sys::Promise, sub_prom: js_sys::Promise) {
    use web_sys::PushSubscription;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen::JsValue;
    use crate::push::{SubInfo, add_subscription};

    log::info!("checking perm");
    let perm = JsFuture::from(perm_prom).await;
    if perm.map(|p| p != "granted").unwrap_or_default() {
        log::warn!("Notification permission not granted");
        return;
    }
    log::info!("granted perm");

    log::info!("checking sub info");
    let Ok(subinfo_js) = JsFuture::from(sub_prom).await else {
        log::warn!("Failed to subscribe");
        return;
    };
    log::info!("got sub info");

    let push_sub = PushSubscription::from(subinfo_js);
    let sub_info_js = push_sub.to_json().unwrap();
    let sub_info: SubInfo = serde_wasm_bindgen::from_value(JsValue::from(sub_info_js)).unwrap();
    add_subscription(sub_info).await.expect("failed to add subscription");
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let push_ctx: PushWorkerCtx = expect_context(); 
    let subscribing = create_rw_signal(false);

    #[cfg(feature = "hydrate")]
    let subscribe_action = create_action(move |(perm_prom, sub_prom): &(js_sys::Promise, js_sys::Promise)| {
        let perf_sub = perform_subscription(perm_prom.clone(), sub_prom.clone());
        async move {
            perf_sub.await;
            subscribing.set(false);
        }
    });
    let on_sub_click = move || {
        subscribing.set(true);
        #[cfg(feature = "hydrate")]
        {
            use web_sys::Notification;
            use web_sys::PushSubscriptionOptionsInit;
            let Ok(perm_prom) = Notification::request_permission() else {
                log::warn!("failed to get notification permission");
                return;
            };

            let worker = push_ctx.worker.get().expect("worker not ready");
            let Ok(pm) = worker.push_manager() else {
                log::warn!("Push manager not available");
                return;
            };
            let mut sub_options = PushSubscriptionOptionsInit::new(); 
            sub_options.user_visible_only(true);
            // TODO: server key
            let sub_prom = pm.subscribe_with_options(&sub_options).unwrap();
            subscribe_action.dispatch((perm_prom, sub_prom));
        }
    };
    let sub_disabled = move || !push_ctx.ready.get() || subscribing();

    let broadcast_action = create_action(move |()| async move {
        broadcast_message("Hello World".into(), "Hello".into()).await.unwrap();
    });
    let broadcast_pending = broadcast_action.pending();

    view! {
        <div class="flex flex-col justify-center items-center min-h-dvh min-w-dvw bg-black text-white">
            <h1>"Web-Push Example"</h1>
            <Show when=subscribing>
                <p>"Subscribing..."</p>
            </Show>
            <Show when=broadcast_pending>
                <p>"Broadcasting..."</p>
            </Show>
            <div class="flex flex-row gap-2 text-black">
                <button class="p-2 bg-gray-50 rounded-md disabled:bg-gray-400" disabled=sub_disabled on:click=move |_| on_sub_click()>"Subscribe"</button>
                <button class="p-2 bg-gray-50 rounded-md disabled:bg-gray-400" disabled=broadcast_pending on:click=move |_| broadcast_action.dispatch(())>"Broadcast"</button>
            </div>
        </div>
    }
}
