use gloo::{events::EventListener, worker::{HandlerId, Worker, WorkerScope}};
use wasm_bindgen::{JsValue, JsCast};
use web_sys::{Notification, NotificationOptions, PushEvent, WorkerGlobalScope};

use super::PushPayload;

pub struct PushWorker {
    #[allow(unused)]
    push_listener: EventListener,
}

impl Worker for PushWorker {
    type Message = ();
    type Input = ();
    type Output = ();

    fn create(_scope: &WorkerScope<Self>) -> Self {
        let this = WorkerGlobalScope::from(JsValue::from(js_sys::global()));
        let push_listener = EventListener::new(&this, "push", move |event| {
            let event = event.unchecked_ref::<PushEvent>();
            let Some(data) = event.data() else {
                log::debug!("received push notif without data");
                return;
            };
            let Ok(payload) = serde_json::from_str::<PushPayload>(&data.text()) else {
                // ideally this should be reported via something like analytics.
                log::error!("received invalid push notification!!");
                return;
            };
            let mut options = NotificationOptions::new();
            options.body(&payload.body);
            let _ = Notification::new_with_options(
                &payload.title,
                &options,
            );
        });

        Self {
            push_listener,
        }
    }

    fn update(&mut self, _scope: &WorkerScope<Self>, _msg: Self::Message) {
        log::warn!("received unsupported update message in push worker");
    }

    fn received(&mut self, _scope: &WorkerScope<Self>, _msg: Self::Input, _id: HandlerId) {
        log::warn!("received unsupported message in push worker");
    }
}