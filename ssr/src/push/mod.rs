use leptos::*;
use serde::{Serialize, Deserialize};

#[cfg(feature = "ssr")]
pub mod sub_kv;


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubInfo {
    pub endpoint: String,
    pub keys: SubKeys,
}

#[cfg(feature = "ssr")]
impl TryFrom<SubInfo> for web_push_native::WebPushBuilder {
    type Error = ServerFnError;

    fn try_from(value: SubInfo) -> Result<Self, Self::Error> {
        use http::Uri;
        use base64ct::{Encoding, Base64UrlUnpadded};
        use web_push_native::{Auth, WebPushBuilder};
        use std::str::FromStr;

        let uri = Uri::from_str(&value.endpoint)?;
        let pubkey = web_push_native::p256::PublicKey::from_sec1_bytes(
            &Base64UrlUnpadded::decode_vec(&value.keys.p256dh)?
        )?;
        let auth = Auth::clone_from_slice(
            &Base64UrlUnpadded::decode_vec(&value.keys.auth)?
        );

        Ok(WebPushBuilder::new(uri, pubkey, auth))
    }
}

#[server]
pub async fn add_subscription(info: SubInfo) -> Result<(), ServerFnError> {
    use sub_kv::SubKV;
    let kv: SubKV = expect_context();
    // WARN: You should verify that subscription info matches server VAPID key
    kv.add_subscription(info).await?;

    Ok(())
}

/// WARN: this is not a secure way to send push notifications
/// anyone can call this endpoint
/// nor is this efficient
#[server]
pub async fn broadcast_message(title: String, body: String) -> Result<(), ServerFnError> {
    // use sub_kv::SubKV;
    // use crate::state::server::VapidKey;
    // use web_push_native::WebPushBuilder;

    // let kv: SubKV = expect_context();
    // let subs = kv.all_subscriptions().await?;

    // let vapid: VapidKey = expect_context();
    // let content = common::PushPayload { title, body };
    // let content_raw = serde_json::to_vec(&content).unwrap();
    // let client: reqwest::Client = expect_context();

    // for sub in subs {
    //     let http_req = WebPushBuilder::try_from(sub)?
    //         .with_vapid(&vapid, "mailto:example@example.com")
    //         .build(content_raw.as_slice())?;

    //     client.execute(http_req.try_into()?).await?;
    // }

    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/setup-firebase-mesaging.js")]
    extern "C" {
        // fn name() -> String;
    
        // type MyClass;
    
        // #[wasm_bindgen(constructor)]
        // fn new() -> MyClass;
    
        // #[wasm_bindgen(method, getter)]
        // fn number(this: &MyClass) -> u32;
        // #[wasm_bindgen(method, setter)]
        // fn set_number(this: &MyClass, number: u32) -> MyClass;
        // #[wasm_bindgen(method)]
        // fn render(this: &MyClass) -> String;

        // #[wasm_bindgen(method)]
        fn get_token() -> String;
    }

    let token = get_token();
    log::info!("got token: {}", token);

    Ok(())
}
