use std::collections;

use anyhow::*;
use enumset::*;

use yew::prelude::Properties;
use yew::prelude::Html;
use yew_router::prelude::Switch as Routable;

use embedded_svc::edge_config::role::Role;

use crate::lambda::Lambda;

#[derive(EnumSetType, Debug, PartialOrd)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum InsertionPoint {
    Drawer,
    Appbar,
}

#[derive(EnumSetType, Debug, PartialOrd)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum Category {
    Header,
    Regular,
    Settings,
}

#[derive(Debug, PartialEq, Clone)]
pub struct APIEndpoint {
    pub uri: String,
    pub headers: collections::HashMap<String, String>,
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct PluginProps<R: Routable + Clone> {
    /// The Switched item representing the active route.
    pub active_route: Option<R>,

    /// The active role in the app.
    pub active_role: Role,

    pub app_bar_renderer: Option<Lambda<(), Html>>,

    /// The API endpoint to be used by all plugins communicating with the server.
    /// Might be `None` if the app is in "demo" mode and is not supposed to file requests to the server.
    pub api_endpoint: Option<APIEndpoint>,
}

impl<R: Routable + Clone> PluginProps<R> {
    pub fn map<F: FnOnce(&R) -> Option<RAPP>, RAPP: Routable + Clone>(&self, mapper: F, api_uri_prefix: &str) -> Result<PluginProps<RAPP>> {
        Ok(PluginProps {
            active_route: match self.active_route {
                Some(ref route) => mapper(route),
                None => None,
            },
            active_role: self.active_role,
            api_endpoint: match self.api_endpoint {
                None => None,
                Some(APIEndpoint{ref uri, ref headers}) => Some(APIEndpoint {
                    uri: crate::api::uri_utils::with_path_segment(uri, api_uri_prefix)?,
                    headers: headers.clone(),
                })
            },
            app_bar_renderer: self.app_bar_renderer.clone(),
        })
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct NavigationPlugin<R: Routable + Clone> {
    pub category: Category,
    pub insertion_point: InsertionPoint,
    pub component: Lambda<PluginProps<R>, Html>,
    pub api_uri_prefix: String,
}

impl<R: 'static + Routable + Clone> NavigationPlugin<R> {
    pub fn map<F: Fn(&RAPP) -> Option<R>, RAPP: Routable + Clone>(&self, mapper: &'static F) -> NavigationPlugin<RAPP> {
        NavigationPlugin {
            category: self.category,
            insertion_point: self.insertion_point,
            component: map(&self.component, self.api_uri_prefix.as_str(), mapper),
            api_uri_prefix: "".into(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ContentPlugin<R: Routable + Clone> {
    pub component: Lambda<PluginProps<R>, Html>,
    pub api_uri_prefix: String,
}

impl<R: 'static + Routable + Clone> ContentPlugin<R> {
    pub fn map<F: Fn(&RAPP) -> Option<R>, RAPP: Routable + Clone>(&self, mapper: &'static F) -> ContentPlugin<RAPP> {
        ContentPlugin {
            component: map(&self.component, self.api_uri_prefix.as_str(), mapper),
            api_uri_prefix: "".into(),
        }
    }
}

fn map<F, R, RAPP>(
    component: &Lambda<PluginProps<R>, Html>,
    api_uri_prefix: &str,
    mapper: &'static F,
) -> Lambda<PluginProps<RAPP>, Html>
where
    F: Fn(&RAPP) -> Option<R>,
    R: 'static + Routable + Clone,
    RAPP: Routable + Clone,
{
    let plugin_component = component.clone();
    let plugin_api_uri_prefix: String = api_uri_prefix.into();

    Lambda::from(move |props: PluginProps<RAPP>|
        plugin_component.call(props.map(mapper, plugin_api_uri_prefix.as_str()).unwrap()))
}
