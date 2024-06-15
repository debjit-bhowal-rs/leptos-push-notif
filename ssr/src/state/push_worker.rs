#[derive(Default, Clone, Copy)]
pub struct PushWorkerCtx {
    #[cfg(feature = "hydrate")]
    pub worker: leptos::RwSignal<Option<web_sys::ServiceWorkerRegistration>>,
    pub ready: leptos::RwSignal<bool>,
}