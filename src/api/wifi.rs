use std::{collections, future::Future, ops::DerefMut, pin::Pin, time::Duration, vec};

use enumset::*;

pub use embedded_svc::wifi::*;
use gloo_timers::future::TimeoutFuture;

#[derive(Clone, PartialEq)]
pub struct Dummy;

impl Dummy {
    fn delay(duration: Duration) -> TimeoutFuture {
        TimeoutFuture::new(duration.as_millis() as _)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WifiAsync;

impl WifiAsync {
    pub async fn get_capabilities(&self) -> Result<EnumSet<Capability>, anyhow::Error> {
        Dummy::delay(Duration::from_millis(5000)).await;

        Ok((Capability::Client | Capability::AccessPoint | Capability::Mixed).into())
    }

    pub async fn get_status(&self) -> Result<Status, anyhow::Error> {
        Dummy::delay(Duration::from_millis(6000)).await;

        Ok(Status(ClientStatus::Stopped, ApStatus::Stopped))
    }

    //async fn scan_n<const N: usize = 20>(&mut self) -> Result<([AccessPointInfo; N], usize)>;

    pub async fn scan(&mut self) -> Result<vec::Vec<AccessPointInfo>, anyhow::Error> {
        Dummy::delay(Duration::from_millis(4000)).await;

        Ok(std::vec![
            AccessPointInfo {
                ssid: "foo".into(),
                bssid: [0; 6],
                channel: 6,
                secondary_channel: SecondaryChannel::None,
                signal_strength: 2,
                protocols: vec!(Protocol::P802D11BGN, Protocol::P802D11LR)
                    .drain(..)
                    .collect(),
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

    pub async fn get_configuration(&self) -> Result<Configuration, anyhow::Error> {
        Dummy::delay(Duration::from_millis(5000)).await;

        Ok(Configuration::Mixed(
            ClientConfiguration {
                ssid: "foo".into(),
                password: "pass".into(),
                ..Default::default()
            },
            AccessPointConfiguration {
                ..Default::default()
            },
        ))
    }

    pub async fn set_configuration(&mut self, _conf: &Configuration) -> Result<(), anyhow::Error> {
        Dummy::delay(Duration::from_millis(500)).await;

        Ok(())
    }
}

// #[derive(Clone, PartialEq)]
// pub struct Rest {
//     uri: String,
//     headers: collections::HashMap<String, String>,
// }

// impl Rest {
//     pub fn new(uri: impl Into<String>, headers: &collections::HashMap<String, String>) -> Self {
//         Rest {
//             uri: uri.into(),
//             headers: headers.clone(),
//         }
//     }

//     fn with_path_segment(&self, segment: impl AsRef<str>) -> anyhow::Result<String> {
//         crate::api::uri_utils::with_path_segment(self.uri.as_str(), segment.as_ref())
//     }
// }

// #[async_trait]
// impl WifiAsync for Rest {
//     type Error = AsStdError<anyhow::Error>;

//     async fn get_capabilities(&self) -> Result<EnumSet<Capability>, Self::Error> {
//         Ok(surf::get(self.with_path_segment("/caps")?)
//             .recv_json()
//             .await
//             .map_err(|e| anyhow::anyhow!(e))?)
//     }

//     async fn get_status(&self) -> Result<Status, Self::Error> {
//         Ok(surf::get(self.with_path_segment("")?)
//             .recv_json()
//             .await
//             .map_err(|e| anyhow::anyhow!(e))?)
//     }

//     async fn scan(&mut self) -> Result<vec::Vec<AccessPointInfo>, Self::Error> {
//         Ok(surf::get(self.with_path_segment("scan")?)
//             .recv_json()
//             .await
//             .map_err(|e| anyhow::anyhow!(e))?)
//     }

//     async fn get_configuration(&self) -> Result<Configuration, Self::Error> {
//         Ok(surf::get(self.with_path_segment("conf")?)
//             .recv_json()
//             .await
//             .map_err(|e| anyhow::anyhow!(e))?)
//     }

//     async fn set_configuration(&mut self, conf: &Configuration) -> Result<(), Self::Error> {
//         let body = surf::Body::from_json(conf).map_err(|e| anyhow::anyhow!(e))?;

//         Ok(surf::post(self.with_path_segment("conf")?)
//             .body(body)
//             .send()
//             .await
//             .map(|_| ())
//             .map_err(|e| anyhow::anyhow!(e))?)
//     }
// }
