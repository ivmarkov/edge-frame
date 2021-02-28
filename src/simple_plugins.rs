use enumset::*;

use yew::prelude::*;
use yew_router::prelude::Switch;

use crate::lambda;
use crate::plugins::*;
use crate::components::router_icon_button::*;
use crate::components::router_list_item::*;

#[derive(PartialEq, Clone, Debug)]
pub struct SimplePlugin<SW: Switch + Clone> {
    pub name: String,

    pub description: Option<String>,
    pub icon: Option<String>,

    pub min_role: Role,

    pub insertion_points: EnumSet<InsertionPoint>,
    pub category: Category,

    pub route: SW,

    pub is_matching_route: lambda::Lambda<SW, bool>,

    pub component: lambda::Lambda<PluginProps<SW>, Html>,
}

impl<SW: 'static + Switch + Clone> From<&SimplePlugin<SW>> for ContentPlugin<SW> {
    fn from(simple_plugin: &SimplePlugin<SW>) -> Self {
        ContentPlugin {
            component: simple_plugin.content_component(),
            api_uri_prefix: "".into(),
        }
    }
}

impl<SW: 'static + Switch + Clone> From<&SimplePlugin<SW>> for std::vec::Vec<NavigationPlugin<SW>> {
    fn from(simple_plugin: &SimplePlugin<SW>) -> Self {
        simple_plugin.insertion_points.iter()
            .map(|registration| NavigationPlugin {
                category: simple_plugin.category,
                insertion_point: registration,
                component: simple_plugin.navigation_component(registration == InsertionPoint::Drawer),
                api_uri_prefix: "".into(),
            })
            .collect()
    }
}

impl<SW: 'static + Switch + Clone> SimplePlugin<SW> {
    pub fn map<F, FR, APPSW>(
        &self, 
        mapper: &'static F, 
        reverse_mapper: &FR) -> SimplePlugin<APPSW> 
        where
            F: Fn(&APPSW) -> Option<SW>, 
            FR: Fn(&SW) -> APPSW, 
            APPSW: 'static + Switch + Clone {
        let plugin_is_matching_route = self.is_matching_route.clone();
        let plugin_component = self.component.clone();

        let is_matching_route = lambda::Lambda::from(move |app_route: APPSW| match mapper(&app_route) {
            Some(plugin_route) => plugin_is_matching_route.call(plugin_route),
            None => false,
        });

        let component = lambda::Lambda::from(move |props: PluginProps<APPSW>| {
            plugin_component.call(PluginProps {
                active_route: match props.active_route {
                    Some(ref app_route) => mapper(app_route),
                    None => None,
                },
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
            route: reverse_mapper(&self.route),
            is_matching_route: is_matching_route,
            component: component,
        }
    }

    fn navigation_component(&self, as_list: bool) -> lambda::Lambda<PluginProps<SW>, Html> {
        let name = self.name.clone();
        let icon = self.icon.clone();
        let route = self.route.clone();
        let min_role = self.min_role;

        lambda::Lambda::from(move |props: PluginProps<SW>|
            if min_role <= props.active_role {
                if as_list {
                    html! {
                        <RouterListItem<SW>
                            text={name.as_str()}
                            icon={icon.clone()}
                            route={route.clone()} 
                            active={false}/> 
                    }
                } else {
                    html! {
                        <RouterIconButton<SW>
                            icon={icon.clone().unwrap_or(String::from("???"))}
                            route={route.clone()}/> 
                    }
                }
            } else {
                html! {}
            })
    }

    fn content_component(&self) -> lambda::Lambda<PluginProps<SW>, Html> {
        let min_role = self.min_role;
        let is_matching_route = self.is_matching_route.clone();
        let component = self.component.clone();
        lambda::Lambda::from(move |props: PluginProps<SW>|
            if 
                min_role <= props.active_role 
                && !props.active_route.is_none() 
                && is_matching_route.call(props.active_route.clone().unwrap()) {
                component.call(props)
            } else {
                html! {}
            })
    }
}
