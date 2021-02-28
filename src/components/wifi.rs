use std::{net::Ipv4Addr, str::FromStr, vec};

use anyhow::*;
use enumset::EnumSet;

use embedded_svc::ipv4;
use embedded_svc::wifi;

use yew::prelude::*;
use yew_router::prelude::Switch;
use yew_mdc::components::*;
use yew_mdc::components::list::*;
use yewtil::future::*;

use lambda::Lambda;

use crate::{lambda, plugins::{Category, InsertionPoint, Role}, simple_plugins::SimplePlugin};

use crate::api;
use crate::plugins::*;

use super::common::*;

#[derive(Debug, Switch, Copy, Clone, PartialEq)]
pub enum Routes {
    #[to = "/"]
    Root,
}

pub fn plugin() -> SimplePlugin<Routes> {
    SimplePlugin {
        name: "WiFi".into(),
        description: Some("A settings user interface for configuring WiFi access".into()),
        icon: Some("wifi".into()),
        min_role: Role::Admin,
        insertion_points: EnumSet::only(InsertionPoint::Drawer).union(EnumSet::only(InsertionPoint::Appbar)),
        category: Category::Settings,
        route: Routes::Root,
        is_matching_route: Lambda::from(|route| route == Routes::Root),
        component: Lambda::from(|props| html! {
            <WiFi with props/>
        }),
    }
}

impl Model<wifi::Configuration> {
    fn client_conf_read<T>(&self, getter: impl FnOnce(&wifi::ClientConfiguration) -> T) -> Option<T> {
        self.0
            .try_borrow()
            .ok()?
            .data_ref()?
            .as_ref()
            .as_client_conf_ref()
            .map(getter)
    }

    fn client_conf_write(&self, updater: impl FnOnce(&mut wifi::ClientConfiguration)) {
        updater(self.0
            .borrow_mut()
            .data_mut()
            .as_mut()
            .as_client_conf_mut());
    }

    fn fixed_settings_read<T>(&self, getter: impl FnOnce(&ipv4::ClientSettings) -> T) -> Option<T> {
        self
            .client_conf_read(|cc| cc.as_ip_conf_ref()?.as_fixed_settings_ref().map(getter))
            .flatten()
    }

    fn fixed_settings_write(&self, updater: impl FnOnce(&mut ipv4::ClientSettings)) {
        self.client_conf_write(|cc| updater(cc.as_ip_conf_mut().as_fixed_settings_mut()));
    }
}

#[derive(Default)]
struct Fields {
    ssid: Field<String>,
    auth_method: Field<wifi::AuthMethod>,
    password: Field<String>,

    subnet: Field<ipv4::Subnet>,
    ip: Field<Ipv4Addr>,
    dns: Field<Optional<Ipv4Addr>>,
    secondary_dns: Field<Optional<Ipv4Addr>>,
}

impl Fields {
    fn load(&mut self) {
        self.ssid.load();
        self.auth_method.load();
        self.password.load();

        self.subnet.load();
        self.ip.load();
        self.dns.load();
        self.secondary_dns.load();
    }
}

pub struct WiFi {
    props: PluginProps<Routes>,
    conf: Model<wifi::Configuration>,

    fields: Fields,

    aps: Loadable<vec::Vec<wifi::AccessPointInfo>>,
    link: ComponentLink<Self>,
}

pub enum Msg {
    GetConfiguration,
    GotConfiguration(Result<wifi::Configuration>),
    GetAccessPoints,
    GotAccessPoints(Result<vec::Vec<wifi::AccessPointInfo>>),

    SSIDChanged(String),
    AuthMethodChanged(String),
    PasswordChanged(String),

    DHCPChanged(bool),
    SubnetChanged(String),
    IpChanged(String),
    DnsChanged(String),
    SecondaryDnsChanged(String),
}

impl WiFi {
    fn create_api(api_endpoint: Option<&APIEndpoint>) -> Box<dyn wifi::WifiAsync> {
        match api_endpoint {
            None => Box::new(api::wifi::Dummy),
            Some(ep) => Box::new(api::wifi::Rest::new(ep.uri.clone(), &ep.headers)),
        }
    }

    fn is_loaded(&self) -> bool {
        self.conf.0.borrow().is_loaded()
    }

    fn is_dhcp(&self) -> bool {
        self.conf.client_conf_read(|cc|
                cc.ip_conf
                    .as_ref()
                    .map_or(
                        true,
                        |ip_conf| match ip_conf {
                            ipv4::ClientConfiguration::DHCP => true,
                            _ => false}))
            .map_or(true, |v| v)
    }

    fn bind_model(&mut self) {
        let model = self.conf.clone();

        let model_r = model.clone();
        let model_w = model.clone();

        self.fields.ssid.bind(
            move || model_r.client_conf_read(|cc| cc.ssid.clone()),
            move |v| model_w.client_conf_write(|cc| cc.ssid = v));

        let model_r = model.clone();
        let model_w = model.clone();

        self.fields.auth_method.bind(
            move || model_r.client_conf_read(|cc| cc.auth_method.clone()),
            move |v| model_w.client_conf_write(|cc| cc.auth_method = v));

        let model_r = model.clone();
        let model_w = model.clone();

        self.fields.password.bind(
            move || model_r.client_conf_read(|cc| cc.password.clone()),
            move |v| model_w.client_conf_write(|cc| cc.password = v));

        let model_r = model.clone();
        let model_w = model.clone();

        self.fields.subnet.bind(
            move || model_r.fixed_settings_read(|cc| cc.subnet.clone()),
            move |v| model_w.fixed_settings_write(|cc| cc.subnet = v));

        let model_r = model.clone();
        let model_w = model.clone();

        self.fields.ip.bind(
            move || model_r.fixed_settings_read(|cc| cc.ip.clone()),
            move |v| model_w.fixed_settings_write(|cc| cc.ip = v));

        let model_r = model.clone();
        let model_w = model.clone();

        self.fields.dns.bind(
            move || model_r.fixed_settings_read(|cc| Optional(cc.dns.clone())),
            move |v| model_w.fixed_settings_write(|cc| cc.dns = v.0));

        let model_r = model.clone();
        let model_w = model.clone();

        self.fields.secondary_dns.bind(
            move || model_r.fixed_settings_read(|cc| Optional(cc.secondary_dns.clone())),
            move |v| model_w.fixed_settings_write(|cc| cc.secondary_dns = v.0));
    }
}

fn as_list<T: Description + ToString + FromStr + IntoDomainIterator>(selected: Option<T>) -> Html {
    html! {
        <List role=list::Role::ListBox>
            {
                for T::iter().map(|v| {
                    let selected = selected
                        .as_ref()
                        .map_or(false, |s| s.to_string() == v.to_string());

                    as_list_item(v, selected)
                })
            }
        </List>
    }
}

fn as_list_item<T: Description + ToString>(item: T, selected: bool) -> Html {
    html! {
        <ListItem
            selected=selected
            tabindex=0
            value=item.to_string()
            role=list_item::Role::Option>
            <ListItemText>{item.get_description()}</ListItemText>
        </ListItem>
    }
}

impl Component for WiFi {
    type Message = Msg;
    type Properties = PluginProps<Routes>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut wifi = Self {
            link,
            conf: Model::new(),
            aps: Default::default(),
            props,
            fields: Default::default()
        };

        wifi.bind_model();

        wifi.link.send_message(Msg::GetConfiguration);

        wifi
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetConfiguration => {
                let api = WiFi::create_api(self.props.api_endpoint.as_ref());

                self.conf.0.borrow_mut().loading();
                self.link.send_future(async move {
                    Msg::GotConfiguration(api.get_configuration().await)
                });

                true
            },
            Msg::GotConfiguration(result) => {
                self.conf.0.borrow_mut().loaded_result(result.map(|data| Editable::new(data)));
                self.fields.load();
                true
            },
            Msg::GetAccessPoints => {
                let mut api = WiFi::create_api(self.props.api_endpoint.as_ref());

                self.aps.loading();
                self.link.send_future(async move {
                    Msg::GotAccessPoints(api.scan().await)
                });

                true
            },
            Msg::GotAccessPoints(result) => {
                self.aps.loaded_result(result);
                self.fields.load();
                true
            },
            Msg::SSIDChanged(value) => {
                self.fields.ssid.update(value);
                true
            },
            Msg::AuthMethodChanged(value) => {
                self.fields.auth_method.update(value);
                true
            },
            Msg::PasswordChanged(value) => {
                self.fields.password.update(value);
                true
            },
            Msg::DHCPChanged(dhcp) => {
                self.conf.client_conf_write(|cc| cc.ip_conf = Some(if dhcp {
                    ipv4::ClientConfiguration::DHCP
                } else {
                    ipv4::ClientConfiguration::Fixed(Default::default())
                }));

                true
            }
            Msg::SubnetChanged(value) => {
                self.fields.subnet.update(value);
                true
            },
            Msg::IpChanged(value) => {
                self.fields.ip.update(value);
                true
            }
            Msg::DnsChanged(value) => {
                self.fields.dns.update(value);
                true
            }
            Msg::SecondaryDnsChanged(value) => {
                self.fields.secondary_dns.update(value);
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.api_endpoint != props.api_endpoint {
            self.conf = Model::new();
            self.aps = Default::default();
            self.props = props;
            self.bind_model();

            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <>
            <div class="mdc-layout-grid" style="width: 50%;">
                <div class="mdc-layout-grid__inner">
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-12">
                        <TextField
                            outlined=true
                            hint="SSID"
                            disabled=!self.is_loaded()
                            onchange=self.link.callback(|value| Msg::SSIDChanged(value))
                            value=self.fields.ssid.get_value_str()
                            valid=Some(self.fields.ssid.is_valid())/>
                        <TextFieldHelperLine validation_msg=true>
                            { self.fields.subnet.get_error_str() }
                        </TextFieldHelperLine>
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-12">
                        <Select
                            outlined=true
                            hint="Authentication"
                            disabled=!self.is_loaded()
                            onchange=self.link.callback(|value| Msg::AuthMethodChanged(value))
                            value=self.fields.auth_method.get_value_str()>
                            {as_list(self.fields.auth_method.get_value())}
                        </Select>
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-12">
                        <TextField
                            outlined=true
                            hint="Password"
                            disabled=!self.is_loaded()
                            onchange=self.link.callback(|value| Msg::PasswordChanged(value))
                            value=self.fields.password.get_value_str()/>
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-12">
                        <yew_mdc::components::Switch
                            label_text="Use DHCP"
                            disabled=!self.is_loaded()
                            onchange=self.link.callback(|state| Msg::DHCPChanged(state))
                            state=self.is_dhcp()/>
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-12">
                        <TextField
                            outlined=true
                            hint="Subnet/Gateway"
                            disabled=!self.is_loaded() || self.is_dhcp()
                            onchange=self.link.callback(|value| Msg::SubnetChanged(value))
                            value=self.fields.subnet.get_value_str()
                            valid=Some(self.fields.subnet.is_valid())/>
                        <TextFieldHelperLine validation_msg=true>
                            { self.fields.subnet.get_error_str() }
                        </TextFieldHelperLine>
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-12">
                        <TextField
                            outlined=true
                            hint="IP"
                            disabled=!self.is_loaded() || self.is_dhcp()
                            onchange=self.link.callback(|value| Msg::IpChanged(value))
                            value=self.fields.ip.get_value_str()/>
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-12">
                        <TextField
                            outlined=true
                            hint="DNS"
                            disabled=!self.is_loaded() || self.is_dhcp()
                            onchange=self.link.callback(|value| Msg::DnsChanged(value))
                            value=self.fields.dns.get_value_str()/>
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-12">
                        <TextField
                            outlined=true
                            hint="Secondary DNS"
                            disabled=!self.is_loaded() || self.is_dhcp()
                            onchange=self.link.callback(|value| Msg::SecondaryDnsChanged(value))
                            value=self.fields.secondary_dns.get_value_str()/>
                    </div>
                </div>
            </div>
            </>
        }
    }
}

