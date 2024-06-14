use std::{fs, path::Path, sync::Arc};

use leptos_router::RouteListing;
use rand::rngs::OsRng;
use axum::extract::FromRef;
use leptos::{LeptosOptions, ServerFnError};
use web_push_native::jwt_simple::algorithms::ES256KeyPair;
use p256::pkcs8::{EncodePrivateKey, LineEnding};

use crate::push::sub_kv::SubKV;

pub type VapidKey = Arc<ES256KeyPair>;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub routes: Vec<RouteListing>,
    pub push_kv: SubKV,
    pub push_client: reqwest::Client,
    pub vapid: VapidKey,
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
            push_client: Default::default(),
            vapid: Arc::new(load_or_gen_vapid_keys()?),
        })
    }
}

pub fn load_or_gen_vapid_keys() -> Result<ES256KeyPair, ServerFnError> {
    let pem_path = Path::new("vapid.pem");
    if pem_path.exists() {
        let pem = fs::read_to_string(pem_path)?;
        return Ok(ES256KeyPair::from_pem(&pem).map_err(|_| ServerFnError::new("Invalid PEM File"))?);
    } 

    log::info!("Vapid keys not found, generating new ones");
    let sk = p256::ecdsa::SigningKey::random(&mut OsRng);
    let pem = sk.to_pkcs8_pem(LineEnding::default()).unwrap();
    let pem_str = pem.as_str();
    fs::write(pem_path, pem_str)?;

    let key = ES256KeyPair::from_pem(&pem).expect("p256 generated invalid key?!");
    Ok(key)
}