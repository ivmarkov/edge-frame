use enumset::*;

use yew::prelude::*;
use yew_router::prelude::Switch as Routed;

use embedded_svc::edge_config::role::Role;

use crate::components::router_icon_button::*;
use crate::components::router_list_item::*;
use crate::lambda;
use crate::plugins::*;

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

    pub component: lambda::Lambda<PluginProps<R>, Html>,
}

impl<R> From<&SimplePlugin<R>> for ContentPlugin<R>
where
    R: Routed + PartialEq + Clone + 'static,
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
    R: Routed + PartialEq + Clone + 'static,
{
    fn from(simple_plugin: &SimplePlugin<R>) -> Self {
        simple_plugin
            .insertion_points
            .iter()
            .map(|registration| NavigationPlugin {
                category: simple_plugin.category,
                insertion_point: registration,
                component: simple_plugin
                    .navigation_component(registration == InsertionPoint::Drawer),
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
    R: Routed + PartialEq + Clone + 'static,
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
                    .navigation_component(insertion_point == InsertionPoint::Drawer),
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

        let component = lambda::Lambda::from(move |props: PluginProps<RAPP>| {
            plugin_component.call(PluginProps {
                active_route: props.active_route == croute,
                active_role: props.active_role,
                api_endpoint: props.api_endpoint,
                app_bar_renderer: props.app_bar_renderer,
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
    R: Routed + PartialEq + Clone + 'static,
{
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = NavigationPlugin<R>> + 'a {
        SimplePluginIterator {
            simple_plugin: self,
            insertion_point_iter: self.insertion_points.iter(),
        }
    }

    fn navigation_component(&self, as_list: bool) -> lambda::Lambda<PluginProps<R>, Html> {
        let name = self.name.clone();
        let icon = self.icon.clone();
        let route = self.route.clone();
        let min_role = self.min_role;

        lambda::Lambda::from(move |props: PluginProps<R>| {
            if min_role <= props.active_role {
                if as_list {
                    html! {
                        <RouterListItem<R>
                            text={name.clone()}
                            icon={icon.clone()}
                            route={route.clone()}
                            active={props.active_route == route}/>
                    }
                } else {
                    html! {
                        <RouterIconButton<R>
                            icon={icon.clone().unwrap_or(String::from("???"))}
                            route={route.clone()}/>
                    }
                }
            } else {
                html! {}
            }
        })
    }

    fn content_component(&self) -> lambda::Lambda<PluginProps<R>, Html> {
        let min_role = self.min_role;
        let route = self.route.clone();
        let component = self.component.clone();
        lambda::Lambda::from(move |props: PluginProps<R>| {
            if min_role <= props.active_role && props.active_route == route {
                component.call(props)
            } else {
                html! {}
            }
        })
    }
}
