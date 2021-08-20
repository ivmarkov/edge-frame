use std::collections;

use enumset::*;

use yew::prelude::Html;
use yew::prelude::Properties;

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
pub struct PluginProps<R>
where
    R: PartialEq + Clone,
{
    /// The Switched item representing the active route.
    pub active_route: R,

    /// The active role in the app.
    pub active_role: Role,

    pub app_bar_renderer: Option<Lambda<(), Html>>,

    /// The API endpoint to be used by all plugins communicating with the server.
    /// Might be `None` if the app is in "demo" mode and is not supposed to file requests to the server.
    pub api_endpoint: Option<APIEndpoint>,
}

impl<RAPP> PluginProps<RAPP>
where
    RAPP: PartialEq + Clone,
{
    pub fn map<F, R>(&self, mapper: F, api_uri_prefix: &str) -> PluginProps<R>
    where
        F: FnOnce(&RAPP) -> R,
        R: PartialEq + Clone,
    {
        PluginProps {
            active_route: mapper(&self.active_route),
            active_role: self.active_role,
            api_endpoint: self.api_endpoint.as_ref().map(
                |APIEndpoint {
                     ref uri,
                     ref headers,
                 }| APIEndpoint {
                    uri: crate::api::uri_utils::with_path_segment(uri, api_uri_prefix).unwrap(),
                    headers: headers.clone(),
                },
            ),
            app_bar_renderer: self.app_bar_renderer.clone(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct NavigationPlugin<R>
where
    R: PartialEq + Clone,
{
    pub category: Category,
    pub insertion_point: InsertionPoint,
    pub component: Lambda<PluginProps<R>, Html>,
    pub api_uri_prefix: String,
}

impl<R> NavigationPlugin<R>
where
    R: PartialEq + Clone,
{
    pub fn map<F, RAPP>(&self, mapper: F) -> NavigationPlugin<RAPP>
    where
        F: Fn(&RAPP) -> R + 'static,
        RAPP: PartialEq + Clone,
        R: 'static,
    {
        NavigationPlugin {
            category: self.category,
            insertion_point: self.insertion_point,
            component: map(&self.component, self.api_uri_prefix.as_str(), mapper),
            api_uri_prefix: "".into(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ContentPlugin<R>
where
    R: PartialEq + Clone,
{
    pub component: Lambda<PluginProps<R>, Html>,
    pub api_uri_prefix: String,
}

impl<R> ContentPlugin<R>
where
    R: PartialEq + Clone,
{
    pub fn map<F, RAPP>(&self, mapper: F) -> ContentPlugin<RAPP>
    where
        F: Fn(&RAPP) -> R + 'static,
        RAPP: PartialEq + Clone,
        R: 'static,
    {
        ContentPlugin {
            component: map(&self.component, self.api_uri_prefix.as_str(), mapper),
            api_uri_prefix: "".into(),
        }
    }
}

fn map<F, R, RAPP>(
    component: &Lambda<PluginProps<R>, Html>,
    api_uri_prefix: &str,
    mapper: F,
) -> Lambda<PluginProps<RAPP>, Html>
where
    F: Fn(&RAPP) -> R + 'static,
    R: PartialEq + Clone + 'static,
    RAPP: PartialEq + Clone,
{
    let plugin_component = component.clone();
    let plugin_api_uri_prefix: String = api_uri_prefix.into();

    Lambda::from(move |props: PluginProps<RAPP>| {
        plugin_component.call(props.map(&mapper, plugin_api_uri_prefix.as_str()))
    })
}
