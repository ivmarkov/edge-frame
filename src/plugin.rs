use std::collections;

use enumset::*;

use yew::prelude::*;
use yew_router::prelude::*;

use embedded_svc::utils::rest::role::Role;

use crate::utils::*;

#[derive(EnumSetType, Debug, PartialOrd, Hash)]
pub enum InsertionPoint {
    Navigation,
    Status,
}

#[derive(EnumSetType, Debug, PartialOrd, Hash)]
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
    pub component: Callback2<PluginProps<R>, Html>,
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
    pub component: Callback2<PluginProps<R>, Html>,
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
    component: &Callback2<PluginProps<R>, Html>,
    api_uri_prefix: &str,
    mapper: F,
) -> Callback2<PluginProps<RAPP>, Html>
where
    F: Fn(&RAPP) -> R + 'static,
    R: PartialEq + Clone + 'static,
    RAPP: PartialEq + Clone,
{
    let plugin_component = component.clone();
    let plugin_api_uri_prefix: String = api_uri_prefix.into();

    Callback2::from(move |props: PluginProps<RAPP>| {
        plugin_component.call(props.map(&mapper, plugin_api_uri_prefix.as_str()))
    })
}

#[derive(PartialEq, Clone, Debug)]
pub struct SimplePlugin<R>
where
    R: PartialEq + Clone,
{
    pub name: String,

    pub description: Option<String>,
    pub icon: Option<String>,

    pub min_role: Role,

    pub insertion_points: EnumSet<InsertionPoint>,
    pub category: Category,

    pub route: R,

    pub component: Callback2<PluginProps<R>, Html>,
}

impl<R> From<&SimplePlugin<R>> for ContentPlugin<R>
where
    R: Routable + PartialEq + Clone + 'static,
{
    fn from(simple_plugin: &SimplePlugin<R>) -> Self {
        ContentPlugin {
            component: simple_plugin.content_component(),
            api_uri_prefix: "".into(),
        }
    }
}

impl<R> From<&SimplePlugin<R>> for std::vec::Vec<NavigationPlugin<R>>
where
    R: Routable + PartialEq + Clone + 'static,
{
    fn from(simple_plugin: &SimplePlugin<R>) -> Self {
        simple_plugin
            .insertion_points
            .iter()
            .map(|registration| NavigationPlugin {
                category: simple_plugin.category,
                insertion_point: registration,
                component: simple_plugin
                    .navigation_component(registration == InsertionPoint::Navigation),
                api_uri_prefix: "".into(),
            })
            .collect()
    }
}

struct SimplePluginIterator<'a, R, I>
where
    R: PartialEq + Clone,
{
    simple_plugin: &'a SimplePlugin<R>,
    insertion_point_iter: I,
}

impl<'a, R, I> Iterator for SimplePluginIterator<'a, R, I>
where
    R: Routable + PartialEq + Clone + 'static,
    I: Iterator<Item = InsertionPoint>,
{
    type Item = NavigationPlugin<R>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(insertion_point) = self.insertion_point_iter.next() {
            Some(NavigationPlugin {
                category: self.simple_plugin.category,
                insertion_point,
                component: self
                    .simple_plugin
                    .navigation_component(insertion_point == InsertionPoint::Navigation),
                api_uri_prefix: "".into(),
            })
        } else {
            None
        }
    }
}

impl SimplePlugin<bool> {
    pub fn map<RAPP>(&self, route: RAPP) -> SimplePlugin<RAPP>
    where
        RAPP: PartialEq + Clone + 'static,
    {
        let plugin_component = self.component.clone();
        let croute = route.clone();

        let component = Callback2::from(move |props: PluginProps<RAPP>| {
            plugin_component.call(PluginProps {
                active_route: props.active_route == croute,
                active_role: props.active_role,
                api_endpoint: props.api_endpoint,
            })
        });

        SimplePlugin {
            name: self.name.clone(),
            description: self.description.clone(),
            icon: self.icon.clone(),
            min_role: self.min_role,
            insertion_points: self.insertion_points.clone(),
            category: self.category,
            route,
            component: component,
        }
    }
}

impl<R> SimplePlugin<R>
where
    R: Routable + PartialEq + Clone + 'static,
{
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = NavigationPlugin<R>> + 'a {
        SimplePluginIterator {
            simple_plugin: self,
            insertion_point_iter: self.insertion_points.iter(),
        }
    }

    fn navigation_component(&self, as_nav_item: bool) -> Callback2<PluginProps<R>, Html> {
        let name = self.name.clone();
        let icon = self.icon.clone();
        let route = self.route.clone();
        let min_role = self.min_role;

        Callback2::from(move |props: PluginProps<R>| {
            if min_role <= props.active_role {
                if as_nav_item {
                    html! {
                        <NavigationItem<R>
                            text={name.clone()}
                            icon={icon.clone()}
                            route={route.clone()}
                            active={props.active_route == route}/>
                    }
                } else {
                    html! {
                        <StatusItem<R>
                            icon={icon.clone().unwrap_or(String::from("???"))}
                            route={route.clone()}/>
                    }
                }
            } else {
                html! {}
            }
        })
    }

    fn content_component(&self) -> Callback2<PluginProps<R>, Html> {
        let min_role = self.min_role;
        let name = self.name.clone();
        let route = self.route.clone();
        let component = self.component.clone();
        Callback2::from(move |props: PluginProps<R>| {
            if min_role <= props.active_role && props.active_route == route {
                html! {
                    <section class="section">
                        <h1 class="title">{name.clone()}</h1>
                        <div>
                            { component.call(props) }
                        </div>
                    </section>
                }
            } else {
                html! {}
            }
        })
    }
}
