use std::collections;

use anyhow::*;
use enumset::*;

use yew::prelude::Properties;
use yew::prelude::Html;
use yew_router::prelude::Switch;

use crate::lambda::Lambda;

#[derive(Debug, EnumSetType)]
pub enum InsertionPoint {
    Drawer,
    Appbar,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum Category {
    Header,
    Regular,
    Settings,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum Role {
    None,
    User,
    Admin
}

impl Default for Role {
    fn default() -> Self {
        Role::None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct APIEndpoint {
    pub uri: String,
    pub headers: collections::HashMap<String, String>,
}

#[derive(Properties, Clone, Default, Debug, PartialEq)]
pub struct PluginProps<SW: Switch + Clone> {
    /// The Switched item representing the active route.
    pub active_route: Option<SW>,

    /// The active role in the app.
    pub active_role: Role,

    /// The API endpoint to be used by all plugins communicating with the server. 
    /// Might be `None` if the app is in "demo" mode and is not supposed to file requests to the server.
    pub api_endpoint: Option<APIEndpoint>,
}

impl<SW: Switch + Clone> PluginProps<SW> {
    pub fn map<F: FnOnce(&SW) -> Option<SWAPP>, SWAPP: Switch + Clone>(&self, mapper: F, api_uri_prefix: &str) -> Result<PluginProps<SWAPP>> {
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
        })
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct NavigationPlugin<SW: Switch + Clone> {
    pub category: Category,
    pub insertion_point: InsertionPoint,
    pub component: Lambda<PluginProps<SW>, Html>,
    pub api_uri_prefix: String,
}

impl<SW: 'static + Switch + Clone> NavigationPlugin<SW> {
    pub fn map<F: Fn(&SWAPP) -> Option<SW>, SWAPP: Switch + Clone>(&self, mapper: &'static F) -> NavigationPlugin<SWAPP> {
        NavigationPlugin {
            category: self.category,
            insertion_point: self.insertion_point,
            component: map(&self.component, self.api_uri_prefix.as_str(), mapper),
            api_uri_prefix: "".into(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ContentPlugin<SW: Switch + Clone> {
    pub component: Lambda<PluginProps<SW>, Html>,
    pub api_uri_prefix: String,
}

impl<SW: 'static + Switch + Clone> ContentPlugin<SW> {
    pub fn map<F: Fn(&SWAPP) -> Option<SW>, SWAPP: Switch + Clone>(&self, mapper: &'static F) -> ContentPlugin<SWAPP> {
        ContentPlugin {
            component: map(&self.component, self.api_uri_prefix.as_str(), mapper),
            api_uri_prefix: "".into(),
        }
    }
}

fn map<F, SW, SWAPP>(
    component: &Lambda<PluginProps<SW>, Html>, 
    api_uri_prefix: &str,
    mapper: &'static F) 
    -> Lambda<PluginProps<SWAPP>, Html> 
    where
        F: Fn(&SWAPP) -> Option<SW>, 
        SW: 'static + Switch + Clone, 
        SWAPP: Switch + Clone {

    let plugin_component = component.clone();
    let plugin_api_uri_prefix: String = api_uri_prefix.into();

    Lambda::from(move |props: PluginProps<SWAPP>|
        plugin_component.call(props.map(mapper, plugin_api_uri_prefix.as_str()).unwrap()))
}
