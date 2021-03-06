use std::{collections, future::Future, ops::DerefMut, pin::Pin, time::Duration, vec};

use async_trait::async_trait;

use anyhow::*;

use enumset::*;

use js_sys::{Function, Promise};

use wasm_bindgen::{JsValue, prelude::Closure};
use wasm_bindgen_futures::JsFuture;

pub use embedded_svc::wifi::*;
use crate::wasm_future::WasmFuture;

#[derive(Clone, PartialEq)]
pub struct Dummy;

impl Dummy {
    fn delay(duration: Duration) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        WasmFuture::new(async move {
            let mut cb = Box::new(|resolve: Function, _reject: Function| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &Closure::once_into_js(move || resolve.call0(&JsValue::null()).unwrap()).into(),
                        duration.as_millis() as i32,
                    )
                    .unwrap();
            });

            match JsFuture::from(Promise::new(cb.deref_mut())).await {
                Ok(_) => Ok(()),
                Err(_) => bail!("Should never happen")
            }
        })
    }
}

#[async_trait]
impl WifiAsync for Dummy {
    type Error = anyhow::Error;

    async fn get_capabilities(&self) -> Result<EnumSet<Capability>, Self::Error> {
        Dummy::delay(Duration::from_millis(500)).await?;

        Ok((Capability::Client | Capability::AccessPoint | Capability::Mixed).into())
    }

    async fn get_status(&self) -> Result<Status> {
        Dummy::delay(Duration::from_millis(500)).await?;

        Ok(Status(ClientStatus::Stopped, ApStatus::Stopped))
    }

    //async fn scan_n<const N: usize = 20>(&mut self) -> Result<([AccessPointInfo; N], usize)>;

    async fn scan(&mut self) -> Result<vec::Vec<AccessPointInfo>, Self::Error> {
        Dummy::delay(Duration::from_millis(4000)).await?;

        Ok(std::vec! [
            AccessPointInfo {
                ssid: "foo".into(),
                bssid: [0; 6],
                channel: 6,
                secondary_channel: SecondaryChannel::None,
                signal_strength: 2,
                protocols: vec!(Protocol::P802D11BGN, Protocol::P802D11LR).drain(..).collect(),
                auth_method: AuthMethod::WPA2Personal,
            },
            AccessPointInfo {
                ssid: "bar".into(),
                bssid: [0; 6],
                channel: 3,
                secondary_channel: SecondaryChannel::None,
                signal_strength: 3,
                protocols: vec!(Protocol::P802D11BGN).drain(..).collect(),
                auth_method: AuthMethod::WEP,
            },
            AccessPointInfo {
                ssid: "open".into(),
                bssid: [0; 6],
                channel: 1,
                secondary_channel: SecondaryChannel::None,
                signal_strength: 3,
                protocols: vec!(Protocol::P802D11BGN).drain(..).collect(),
                auth_method: AuthMethod::None,
            },
        ])
    }

    async fn get_configuration(&self) -> Result<Configuration, Self::Error> {
        Dummy::delay(Duration::from_millis(500)).await?;

        Ok(Configuration::Client(ClientConfiguration {
            ssid: "foo".into(),
            password: "pass".into(),
            ..Default::default()
        }))
    }

    async fn set_configuration(&mut self, _conf: &Configuration) -> Result<(), Self::Error> {
        Dummy::delay(Duration::from_millis(500)).await?;

        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub struct Rest {
    uri: String,
    headers: collections::HashMap<String, String>,
}

impl Rest {
    pub fn new(uri: impl Into<String>, headers: &collections::HashMap<String, String>) -> Self {
        Rest {
            uri: uri.into(),
            headers: headers.clone(),
        }
    }

    fn with_path_segment(&self, segment: impl AsRef<str>) -> Result<String> {
        crate::api::uri_utils::with_path_segment(self.uri.as_str(), segment.as_ref())
    }
}

 #[async_trait]
impl WifiAsync for Rest {
    type Error = anyhow::Error;

    async fn get_capabilities(&self) -> Result<EnumSet<Capability>, Self::Error> {
        surf::get(self.with_path_segment("/caps")?)
            .recv_json()
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn get_status(&self) -> Result<Status, Self::Error> {
        surf::get(self.with_path_segment("")?)
            .recv_json()
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn scan(&mut self) -> Result<vec::Vec<AccessPointInfo>, Self::Error> {
        surf::get(self.with_path_segment("scan")?)
            .recv_json()
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn get_configuration(&self) -> anyhow::Result<Configuration, Self::Error> {
        surf::get(self.with_path_segment("conf")?)
            .recv_json()
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn set_configuration(&mut self, conf: &Configuration) -> Result<(), Self::Error> {
        let body = surf::Body::from_json(conf)
            .map_err(|e| anyhow!(e))?;

        surf::post(self.with_path_segment("conf")?)
            .body(body)
            .send()
            .await
            .map(|_| ())
            .map_err(|e| anyhow!(e))
    }
}
