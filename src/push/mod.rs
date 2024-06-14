use leptos::{expect_context, server, ServerFnError};
use serde::{Serialize, Deserialize};
use sub_kv::SubKV;
use web_push::{ContentEncoding, HyperWebPushClient, PartialVapidSignatureBuilder, SubscriptionInfo, WebPushClient, WebPushMessageBuilder};

#[cfg(feature = "hydrate")]
pub mod worker;
#[cfg(feature = "ssr")]
pub mod sub_kv;

#[derive(Serialize, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
}

#[server]
pub async fn add_subscription(info: SubscriptionInfo) -> Result<(), ServerFnError> {
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
    let kv: SubKV = expect_context();
    let subs = kv.all_subscriptions().await?;

    let sigb: PartialVapidSignatureBuilder = expect_context();
    let content = PushPayload { title, body };
    let content_raw = serde_json::to_vec(&content).unwrap();
    let client: HyperWebPushClient = expect_context();

    for sub in subs {
        let sig = sigb.clone().add_sub_info(&sub).build()?;

        let mut builder = WebPushMessageBuilder::new(&sub);
        builder.set_payload(ContentEncoding::Aes128Gcm, &content_raw);
        builder.set_vapid_signature(sig);

        client.send(builder.build()?).await?;
    }

    Ok(())
}
