use std::{fs::{self, File}, io::Cursor, path::Path};

use leptos_router::RouteListing;
use rand::rngs::OsRng;
use axum::extract::FromRef;
use leptos::{LeptosOptions, ServerFnError};
use web_push::{HyperWebPushClient, PartialVapidSignatureBuilder, VapidSignatureBuilder};
use p256::pkcs8::{EncodePrivateKey, LineEnding};

use crate::push::sub_kv::SubKV;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub routes: Vec<RouteListing>,
    pub push_kv: SubKV,
    pub push_client: HyperWebPushClient,
    pub sig_b: PartialVapidSignatureBuilder,
}

impl AppState {
    pub fn new(
        leptos_options: LeptosOptions,
        routes: Vec<RouteListing>,
    ) -> Result<Self, ServerFnError> {
        Ok(Self {
            leptos_options,
            routes,
            push_kv: SubKV::new()?,
            push_client: HyperWebPushClient::default(),
            sig_b: load_or_gen_vapid_keys()?,
        })
    }
}

pub fn load_or_gen_vapid_keys() -> Result<PartialVapidSignatureBuilder, ServerFnError> {
    let pem_path = Path::new("vapid.pem");
    if pem_path.exists() {
        let f = File::open(pem_path)?;
        return Ok(VapidSignatureBuilder::from_pem_no_sub(&f)?);
    } 

    log::info!("Vapid keys not found, generating new ones");
    let sk = p256::ecdsa::SigningKey::random(&mut OsRng);
    let pem = sk.to_pkcs8_pem(LineEnding::default()).unwrap();
    let pem_slice: &[u8] = pem.as_ref();
    fs::write(pem_path, pem_slice)?;

    let mut pem_cursor = Cursor::new(pem_slice);
    let key = VapidSignatureBuilder::from_pem_no_sub(&mut pem_cursor).expect("p256 generated invalid key?!");
    Ok(key)
}