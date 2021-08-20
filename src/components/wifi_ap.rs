use std::{cell::RefCell, convert::TryFrom, net::Ipv4Addr, rc::Rc, str::FromStr, vec};

use anyhow::*;

use enumset::EnumSet;

use embedded_svc::wifi;
use embedded_svc::{ipv4, wifi::AuthMethod};

use yew::prelude::*;

use yewtil::future::*;

use embedded_svc::edge_config::role::Role;

use material_yew::select::ListIndex;
use material_yew::select::SelectedDetail;
use material_yew::*;

use lambda::Lambda;

use crate::{
    lambda,
    plugins::{Category, InsertionPoint},
    simple_plugins::SimplePlugin,
};

use crate::api;
use crate::plugins::*;

use super::common::*;

pub fn plugin() -> SimplePlugin<bool> {
    SimplePlugin {
        name: "WiFi Ap".into(),
        description: Some("A settings user interface for configuring WiFi Access Point".into()),
        icon: Some("wifi".into()),
        min_role: Role::Admin,
        insertion_points: EnumSet::only(InsertionPoint::Drawer)
            .union(EnumSet::only(InsertionPoint::Appbar)),
        category: Category::Settings,
        route: true,
        component: Lambda::from(|props| {
            html! {
                <WiFiAp with props/>
            }
        }),
    }
}

impl Loadable<Editable<wifi::Configuration>> {
    fn ap_conf(&self) -> Option<&wifi::AccessPointConfiguration> {
        self.data_ref()?.as_ref().as_ap_conf_ref()
    }

    fn ap_conf_mut(&mut self) -> &mut wifi::AccessPointConfiguration {
        self.data_mut().as_mut().as_ap_conf_mut()
    }

    fn ap_ip_conf(&self) -> Option<&ipv4::RouterConfiguration> {
        self.ap_conf()?.as_ip_conf_ref()
    }

    fn ap_ip_conf_mut(&mut self) -> &mut ipv4::RouterConfiguration {
        self.ap_conf_mut().as_ip_conf_mut()
    }
}

impl Model<Editable<wifi::Configuration>> {
    fn bind_model_wifi_ap<T: Clone + 'static>(
        &self,
        f: &mut Field<T>,
        getter: fn(&wifi::AccessPointConfiguration) -> &T,
        updater: fn(&mut wifi::AccessPointConfiguration) -> &mut T,
    ) {
        let model_r = self.clone();
        let model_w = self.clone();

        f.bind(
            move || model_r.0.borrow().ap_conf().map(getter).map(Clone::clone),
            move |v| *updater(model_w.0.borrow_mut().ap_conf_mut()) = v,
        );
    }

    fn bind_model_ip_ap<Q, T>(
        &self,
        f: &mut Field<Q>,
        getter: fn(&ipv4::RouterConfiguration) -> &T,
        updater: fn(&mut ipv4::RouterConfiguration) -> &mut T,
    ) where
        T: From<Q> + Clone + 'static,
        Q: From<T> + Clone + 'static,
    {
        let model_r = self.clone();
        let model_w = self.clone();

        f.bind(
            move || {
                model_r
                    .0
                    .borrow()
                    .ap_ip_conf()
                    .map(getter)
                    .map(Clone::clone)
                    .map(Into::into)
            },
            move |v| *updater(model_w.0.borrow_mut().ap_ip_conf_mut()) = v.into(),
        );
    }
}

impl Loadable<wifi::Status> {
    fn ap_status_str(&self) -> &'static str {
        if !self.is_loaded() {
            return "Waiting for status info...";
        }

        let status = self.data_ref().unwrap();

        match &status.1 {
            wifi::ApStatus::Stopped => "Stopped",
            wifi::ApStatus::Starting => "Starting...",
            wifi::ApStatus::Started(ref ss) => match ss {
                wifi::ApIpStatus::Disabled => "Disabled",
                wifi::ApIpStatus::Waiting => "Waiting for IP...",
                wifi::ApIpStatus::Done => "Connected",
            },
        }
    }
}

#[derive(Default)]
struct Fields {
    ssid: Field<String>,
    auth_method: Field<wifi::AuthMethod>,
    password: Field<String>,
    password_confirmed: Field<String>,

    subnet: Field<ipv4::Subnet>,
    dns: Field<Optional<Ipv4Addr>>,
    secondary_dns: Field<Optional<Ipv4Addr>>,
}

impl Fields {
    fn load(&mut self) {
        self.ssid.load();
        self.auth_method.load();
        self.password.load();
        self.password_confirmed.load();

        self.subnet.load();
        self.dns.load();
        self.secondary_dns.load();
    }
}

pub struct WiFiAp {
    props: PluginProps<bool>,
    conf: Model<Editable<wifi::Configuration>>,

    fields: Fields,

    password_confirmed: Rc<RefCell<String>>,

    status: Model<wifi::Status>,

    link: ComponentLink<Self>,
}

pub enum Msg {
    GetConfiguration,
    GotConfiguration(Result<wifi::Configuration>),
    GetStatus,
    GotStatus(Result<wifi::Status>),

    SSIDChanged(String),
    SSIDHiddenChanged(bool),
    AuthMethodChanged(AuthMethod),
    PasswordChanged(String),
    PasswordConfirmedChanged(String),

    SubnetChanged(String),
    DHCPEnabledChanged(bool),
    DnsChanged(String),
    SecondaryDnsChanged(String),

    None,
}

impl WiFiAp {
    fn create_api(
        api_endpoint: Option<&APIEndpoint>,
    ) -> Box<dyn wifi::WifiAsync<Error = anyhow::Error>> {
        match api_endpoint {
            None => Box::new(api::wifi::Dummy),
            Some(ep) => Box::new(api::wifi::Rest::new(ep.uri.clone(), &ep.headers)),
        }
    }

    fn is_loaded(&self) -> bool {
        self.conf.0.borrow().is_loaded() && self.status.0.borrow().is_loaded()
    }

    fn bind_model(&mut self) {
        self.conf.bind_model_wifi_ap(
            &mut self.fields.ssid,
            |conf| &conf.ssid,
            |conf| &mut conf.ssid,
        );

        self.conf.bind_model_wifi_ap(
            &mut self.fields.auth_method,
            |conf| &conf.auth_method,
            |conf| &mut conf.auth_method,
        );

        self.conf.bind_model_wifi_ap(
            &mut self.fields.password,
            |conf| &conf.password,
            |conf| &mut conf.password,
        );

        let password_confirmed_g = self.password_confirmed.clone();
        let password_confirmed_u = self.password_confirmed.clone();

        self.fields.password_confirmed.bind(
            move || Some(password_confirmed_g.borrow().clone()),
            move |value| *password_confirmed_u.borrow_mut() = value,
        );

        self.conf.bind_model_ip_ap(
            &mut self.fields.subnet,
            |settings| &settings.subnet,
            |settings| &mut settings.subnet,
        );

        self.conf.bind_model_ip_ap(
            &mut self.fields.dns,
            |settings| &settings.dns,
            |settings| &mut settings.dns,
        );

        self.conf.bind_model_ip_ap(
            &mut self.fields.secondary_dns,
            |settings| &settings.secondary_dns,
            |settings| &mut settings.secondary_dns,
        );
    }
}

fn as_list<T: Description + ToString + FromStr + IntoDomainIterator>(selected: Option<T>) -> Html {
    html! {
        <>
            {
                for T::iter().map(|v| {
                    let selected = selected
                        .as_ref()
                        .map_or(false, |s| s.to_string() == v.to_string());

                    as_list_item(v, selected)
                })
            }
        </>
    }
}

fn as_list_item<T: Description + ToString>(item: T, selected: bool) -> Html {
    html! {
        <MatListItem
            selected=selected
            tabindex=0
            value=item.to_string()
        >
            {item.get_description()}
        </MatListItem>
    }
}

impl Component for WiFiAp {
    type Message = Msg;
    type Properties = PluginProps<bool>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut wifi = Self {
            link,
            conf: Model::new(),
            status: Default::default(),
            props,
            fields: Default::default(),
            password_confirmed: Rc::new(RefCell::new("".into())),
        };

        wifi.bind_model();

        wifi.link.send_message(Msg::GetConfiguration);
        wifi.link.send_message(Msg::GetStatus);

        wifi
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetConfiguration => {
                let api = Self::create_api(self.props.api_endpoint.as_ref());

                self.conf.0.borrow_mut().loading();
                self.link.send_future(async move {
                    Msg::GotConfiguration(api.get_configuration().await)
                });

                true
            }
            Msg::GotConfiguration(result) => {
                self.conf
                    .0
                    .borrow_mut()
                    .loaded_result(result.map(|data| Editable::new(data)));
                self.fields.load();
                true
            }
            Msg::GetStatus => {
                let api = Self::create_api(self.props.api_endpoint.as_ref());

                self.status.0.borrow_mut().loading();
                self.link
                    .send_future(async move { Msg::GotStatus(api.get_status().await) });

                true
            }
            Msg::GotStatus(result) => {
                self.status.0.borrow_mut().loaded_result(result);
                self.fields.load();
                true
            }
            Msg::SSIDChanged(value) => {
                self.fields.ssid.update(value);
                true
            }
            Msg::SSIDHiddenChanged(value) => {
                self.conf.0.borrow_mut().ap_conf_mut().ssid_hidden = value;
                true
            }
            Msg::AuthMethodChanged(value) => {
                self.fields.auth_method.update(value.to_string());
                true
            }
            Msg::PasswordChanged(value) => {
                self.fields.password.update(value);
                true
            }
            Msg::PasswordConfirmedChanged(value) => {
                self.fields.password_confirmed.update(value);
                true
            }
            Msg::SubnetChanged(value) => {
                self.fields.subnet.update(value);
                true
            }
            Msg::DHCPEnabledChanged(value) => {
                self.conf
                    .0
                    .borrow_mut()
                    .ap_conf_mut()
                    .as_ip_conf_mut()
                    .dhcp_enabled = value;
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
            Msg::None => false,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.api_endpoint != props.api_endpoint {
            self.conf = Model::new();
            self.props = props;
            self.bind_model();

            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        self.view_configuration()
    }
}

impl WiFiAp {
    fn view_configuration(&self) -> Html {
        // TODO validity_transform={Some(MatTextField::validity_transform(|_, _| *ValidityState::new().set_bad_input(self.fields.ssid.is_valid())))}

        html! {
            <>
            {self.props.app_bar_renderer.as_ref().unwrap().call(())}
            <CenteredGrid>
                <div class="mdc-layout-grid__inner">
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                        { self.status.0.borrow().ap_status_str() }
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                        <MatTextField
                            outlined=true
                            label="SSID"
                            disabled=!self.is_loaded()
                            value=self.fields.ssid.get_value_str().to_owned()
                            oninput=self.link.callback(|id: InputData| Msg::SSIDChanged(id.value))
                            validate_on_initial_render=true
                            auto_validate=true
                            helper={ self.fields.ssid.get_error_str() }
                        />
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                        <span>{"Hide SSID"}</span>
                        <MatSwitch
                            disabled=!self.is_loaded()
                            onchange=self.link.callback(|state| Msg::SSIDHiddenChanged(state))
                            checked=self.conf.0.borrow().ap_conf().map(|a| a.ssid_hidden).unwrap_or(false)
                        />
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                        <MatSelect
                            outlined=true
                            label="Authentication"
                            disabled=!self.is_loaded()
                            value=self.fields.auth_method.get_value_str().to_owned()
                            onselected=self.link.callback(|sd: SelectedDetail| match sd.index {
                                ListIndex::Single(Some(index)) => Msg::AuthMethodChanged(AuthMethod::try_from(index as u8).unwrap()),
                                _ => Msg::None,
                            })
                        >
                            {as_list(self.fields.auth_method.get_value())}
                        </MatSelect>
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    {
                        if Some(wifi::AuthMethod::None) != self.fields.auth_method.get_value() {
                            html! {
                                <>
                                <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                                </div>
                                <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                                    <MatTextField
                                        outlined=true
                                        label={
                                            if Some(wifi::AuthMethod::WEP) == self.fields.auth_method.get_value() {
                                                "Key"
                                            } else {
                                                "Password"
                                            }
                                        }
                                        disabled=!self.is_loaded()
                                        value=self.fields.password.get_value_str().to_owned()
                                        oninput=self.link.callback(|id: InputData| Msg::PasswordChanged(id.value))
                                        validate_on_initial_render=true
                                        auto_validate=true
                                        helper={ self.fields.password.get_error_str() }
                                    />
                                </div>
                                <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                                </div>
                                <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                                </div>
                                <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                                    <MatTextField
                                        outlined=true
                                        label={
                                            if Some(wifi::AuthMethod::WEP) == self.fields.auth_method.get_value() {
                                                "Confirm Key"
                                            } else {
                                                "Confirm Password"
                                            }
                                        }
                                        disabled=!self.is_loaded()
                                        value=self.fields.password_confirmed.get_value_str().to_owned()
                                        oninput=self.link.callback(|id: InputData| Msg::PasswordConfirmedChanged(id.value))
                                        validate_on_initial_render=true
                                        auto_validate=true
                                        helper={ self.fields.password_confirmed.get_error_str() }
                                    />
                                </div>
                                <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                                </div>
                                </>
                            }
                        } else {
                            html! {}
                        }
                    }
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                        <MatTextField
                            outlined=true
                            label="Subnet/Gateway"
                            disabled=!self.is_loaded()
                            value=self.fields.subnet.get_value_str().to_owned()
                            oninput=self.link.callback(|id: InputData| Msg::SubnetChanged(id.value))
                            validate_on_initial_render=true
                            auto_validate=true
                            helper={ self.fields.subnet.get_error_str() }
                        />
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                        <span>{"DHCP"}</span>
                        <MatSwitch
                            disabled=!self.is_loaded()
                            onchange=self.link.callback(|state| Msg::DHCPEnabledChanged(state))
                            checked=self.conf.0.borrow().ap_ip_conf().map(|i| i.dhcp_enabled).unwrap_or(false)
                        />
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                        <MatTextField
                            outlined=true
                            label="DNS"
                            disabled=!self.is_loaded()
                            value=self.fields.dns.get_value_str().to_owned()
                            oninput=self.link.callback(|id: InputData| Msg::DnsChanged(id.value))
                            validate_on_initial_render=true
                            auto_validate=true
                            helper={ self.fields.dns.get_error_str() }
                        />
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-10" style="text-align: center;">
                        <MatTextField
                            outlined=true
                            label="Secondary DNS"
                            disabled=!self.is_loaded()
                            value=self.fields.secondary_dns.get_value_str().to_owned()
                            oninput=self.link.callback(|id: InputData| Msg::SecondaryDnsChanged(id.value))
                            validate_on_initial_render=true
                            auto_validate=true
                            helper={ self.fields.secondary_dns.get_error_str() }
                        />
                    </div>
                    <div class="mdc-layout-grid__cell mdc-layout-grid__cell--span-1">
                    </div>
                </div>
            </CenteredGrid>
            </>
        }
    }
}
