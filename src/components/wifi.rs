use std::{cell::RefCell, convert::TryFrom, net::Ipv4Addr, rc::Rc, str::FromStr, vec};

use anyhow::*;

use enumset::EnumSet;

use embedded_svc::wifi;
use embedded_svc::{
    ipv4,
    wifi::{AuthMethod, TransitionalState},
};

use yew::prelude::*;

use yewtil::future::*;

use embedded_svc::edge_config::role::Role;

use material_yew::list::GraphicType;
use material_yew::list::RequestSelectedDetail;
use material_yew::select::ListIndex;
use material_yew::select::SelectedDetail;
use material_yew::top_app_bar::*;
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum PluginBehavior {
    STA,
    AP,
    Mixed,
}

pub fn plugin(behavior: PluginBehavior) -> SimplePlugin<bool> {
    SimplePlugin {
        name: "WiFi".into(),
        description: Some(
            match behavior {
                PluginBehavior::STA => "A settings user interface for configuring WiFi access",
                PluginBehavior::AP => "A settings user interface for configuring WiFi Access Point",
                PluginBehavior::Mixed => {
                    "A settings user interface for configuring WiFi Access Point and STA"
                }
            }
            .into(),
        ),
        icon: Some("wifi".into()),
        min_role: Role::Admin,
        insertion_points: EnumSet::only(InsertionPoint::Drawer)
            .union(EnumSet::only(InsertionPoint::Appbar)),
        category: Category::Settings,
        route: true,
        component: Lambda::from(move |plugin_props: PluginProps<bool>| {
            let props = Props {
                behavior,
                api_endpoint: plugin_props.api_endpoint,
                app_bar_renderer: plugin_props.app_bar_renderer,
            };

            html! {
                <WiFi with props/>
            }
        }),
    }
}

impl Loadable<Editable<wifi::Configuration>> {
    fn client_conf(&self) -> Option<&wifi::ClientConfiguration> {
        self.data_ref()?.as_ref().as_client_conf_ref()
    }

    fn client_conf_mut(&mut self) -> &mut wifi::ClientConfiguration {
        self.data_mut().as_mut().as_client_conf_mut()
    }

    fn client_ip_conf(&self) -> Option<&ipv4::ClientConfiguration> {
        self.client_conf()?.as_ip_conf_ref()
    }

    fn client_ip_settings(&self) -> Option<&ipv4::ClientSettings> {
        self.client_ip_conf()?.as_fixed_settings_ref()
    }

    fn client_ip_settings_mut(&mut self) -> &mut ipv4::ClientSettings {
        self.client_conf_mut()
            .as_ip_conf_mut()
            .as_fixed_settings_mut()
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
    fn bind_model_wifi<T: Clone + 'static>(
        &self,
        f: &mut Field<T>,
        getter: fn(&wifi::ClientConfiguration) -> &T,
        updater: fn(&mut wifi::ClientConfiguration) -> &mut T,
    ) {
        let model_r = self.clone();
        let model_w = self.clone();

        f.bind(
            move || {
                model_r
                    .0
                    .borrow()
                    .client_conf()
                    .map(getter)
                    .map(Clone::clone)
            },
            move |v| *updater(model_w.0.borrow_mut().client_conf_mut()) = v,
        );
    }

    fn bind_model_ip<Q, T>(
        &self,
        status: &Model<wifi::Status>,
        f: &mut Field<Q>,
        getter: fn(&ipv4::ClientSettings) -> &T,
        updater: fn(&mut ipv4::ClientSettings) -> &mut T,
    ) where
        T: From<Q> + Clone + 'static,
        Q: From<T> + Clone + 'static,
    {
        let model_r = self.clone();
        let model_w = self.clone();
        let status_model = status.clone();

        f.bind(
            move || {
                model_r
                    .0
                    .borrow()
                    .client_ip_settings()
                    .or(status_model.0.borrow().client_ip_settings())
                    .map(getter)
                    .map(Clone::clone)
                    .map(Into::into)
            },
            move |v| *updater(model_w.0.borrow_mut().client_ip_settings_mut()) = v.into(),
        );
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
    fn client_ip_settings(&self) -> Option<&ipv4::ClientSettings> {
        self.data_ref()?
            .0
            .get_operating()?
            .get_operating()?
            .get_operating()
    }

    fn client_status_str(&self) -> &'static str {
        if !self.is_loaded() {
            return "Waiting for status info...";
        }

        let status = self.data_ref().unwrap();

        match &status.0 {
            wifi::ClientStatus::Stopped => "Stopped",
            wifi::ClientStatus::Starting => "Starting...",
            wifi::ClientStatus::Started(ref ss) => match ss {
                wifi::ClientConnectionStatus::Disconnected => "Disconnected",
                wifi::ClientConnectionStatus::Connecting => "Connecting...",
                wifi::ClientConnectionStatus::Connected(ref cc) => match cc {
                    wifi::ClientIpStatus::Disabled => "Connected (IP disabled)",
                    wifi::ClientIpStatus::Waiting => "Waiting for IP...",
                    wifi::ClientIpStatus::Done(_) => "Connected",
                },
            },
        }
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

#[derive(Default)]
struct ApFields {
    ssid: Field<String>,
    auth_method: Field<wifi::AuthMethod>,
    password: Field<String>,
    password_confirmed: Field<String>,

    subnet: Field<ipv4::Subnet>,
    dns: Field<Optional<Ipv4Addr>>,
    secondary_dns: Field<Optional<Ipv4Addr>>,
}

impl ApFields {
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

#[derive(Properties, Clone, Debug, PartialEq)]
pub struct Props {
    pub behavior: PluginBehavior,
    pub app_bar_renderer: Option<Lambda<(), Html>>,
    pub api_endpoint: Option<APIEndpoint>,
}

pub struct WiFi {
    props: Props,
    conf: Model<Editable<wifi::Configuration>>,

    fields: Fields,
    ap_fields: ApFields,

    status: Model<wifi::Status>,

    aps: Loadable<vec::Vec<wifi::AccessPointInfo>>,

    access_points_shown: bool,
    password_confirmed: Rc<RefCell<String>>,

    link: ComponentLink<Self>,
}

pub enum Msg {
    GetConfiguration,
    GotConfiguration(Result<wifi::Configuration>),
    GetStatus,
    GotStatus(Result<wifi::Status>),

    GetAccessPoints,
    GotAccessPoints(Result<vec::Vec<wifi::AccessPointInfo>>),

    ShowAccessPoints,
    ShowConfiguration(Option<(String, AuthMethod)>),

    SSIDChanged(String),
    AuthMethodChanged(AuthMethod),
    PasswordChanged(String),

    DHCPChanged(bool),
    SubnetChanged(String),
    IpChanged(String),
    DnsChanged(String),
    SecondaryDnsChanged(String),

    ApSSIDChanged(String),
    ApSSIDHiddenChanged(bool),
    ApAuthMethodChanged(AuthMethod),
    ApPasswordChanged(String),
    ApPasswordConfirmedChanged(String),

    ApSubnetChanged(String),
    ApDHCPEnabledChanged(bool),
    ApDnsChanged(String),
    ApSecondaryDnsChanged(String),

    None,
}

impl WiFi {
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

    fn is_dhcp(&self) -> bool {
        match self.conf.0.borrow().client_ip_conf() {
            Some(ipv4::ClientConfiguration::DHCP) | None => true,
            _ => false,
        }
    }
}

impl WiFi {
    fn bind_model(&mut self) {
        self.conf.bind_model_wifi(
            &mut self.fields.ssid,
            |conf| &conf.ssid,
            |conf| &mut conf.ssid,
        );

        self.conf.bind_model_wifi(
            &mut self.fields.auth_method,
            |conf| &conf.auth_method,
            |conf| &mut conf.auth_method,
        );

        self.conf.bind_model_wifi(
            &mut self.fields.password,
            |conf| &conf.password,
            |conf| &mut conf.password,
        );

        self.conf.bind_model_ip(
            &self.status,
            &mut self.fields.subnet,
            |settings| &settings.subnet,
            |settings| &mut settings.subnet,
        );

        self.conf.bind_model_ip(
            &self.status,
            &mut self.fields.ip,
            |settings| &settings.ip,
            |settings| &mut settings.ip,
        );

        self.conf.bind_model_ip(
            &self.status,
            &mut self.fields.dns,
            |settings| &settings.dns,
            |settings| &mut settings.dns,
        );

        self.conf.bind_model_ip(
            &self.status,
            &mut self.fields.secondary_dns,
            |settings| &settings.secondary_dns,
            |settings| &mut settings.secondary_dns,
        );
    }
}

impl WiFi {
    fn bind_model_ap(&mut self) {
        self.conf.bind_model_wifi_ap(
            &mut self.ap_fields.ssid,
            |conf| &conf.ssid,
            |conf| &mut conf.ssid,
        );

        self.conf.bind_model_wifi_ap(
            &mut self.ap_fields.auth_method,
            |conf| &conf.auth_method,
            |conf| &mut conf.auth_method,
        );

        self.conf.bind_model_wifi_ap(
            &mut self.ap_fields.password,
            |conf| &conf.password,
            |conf| &mut conf.password,
        );

        let password_confirmed_g = self.password_confirmed.clone();
        let password_confirmed_u = self.password_confirmed.clone();

        self.ap_fields.password_confirmed.bind(
            move || Some(password_confirmed_g.borrow().clone()),
            move |value| *password_confirmed_u.borrow_mut() = value,
        );

        self.conf.bind_model_ip_ap(
            &mut self.ap_fields.subnet,
            |settings| &settings.subnet,
            |settings| &mut settings.subnet,
        );

        self.conf.bind_model_ip_ap(
            &mut self.ap_fields.dns,
            |settings| &settings.dns,
            |settings| &mut settings.dns,
        );

        self.conf.bind_model_ip_ap(
            &mut self.ap_fields.secondary_dns,
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

impl Component for WiFi {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut wifi = Self {
            props,
            conf: Model::new(),
            fields: Default::default(),
            ap_fields: Default::default(),
            aps: Default::default(),
            status: Default::default(),
            password_confirmed: Rc::new(RefCell::new("".into())),
            access_points_shown: false,
            link,
        };

        wifi.bind_model();
        wifi.bind_model_ap();

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
                self.ap_fields.load();
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
                self.ap_fields.load();
                true
            }
            Msg::GetAccessPoints => {
                let mut api = Self::create_api(self.props.api_endpoint.as_ref());

                self.aps.loading();
                self.link
                    .send_future(async move { Msg::GotAccessPoints(api.scan().await) });

                true
            }
            Msg::GotAccessPoints(result) => {
                self.aps.loaded_result(result);
                self.fields.load();
                self.ap_fields.load();
                true
            }
            Msg::ShowAccessPoints => {
                if !self.access_points_shown {
                    self.access_points_shown = true;

                    if !self.aps.is_loaded() {
                        self.link.send_message(Msg::GetAccessPoints);
                    }

                    true
                } else {
                    false
                }
            }
            Msg::ShowConfiguration(data) => {
                if self.access_points_shown {
                    self.access_points_shown = false;
                    if let Some((ssid, auth_method)) = data {
                        self.conf.0.borrow_mut().client_conf_mut().ssid = ssid;
                        self.conf.0.borrow_mut().client_conf_mut().auth_method = auth_method;

                        self.fields.ssid.load();
                        self.fields.auth_method.load();
                    }
                    true
                } else {
                    false
                }
            }
            Msg::SSIDChanged(value) => {
                self.fields.ssid.update(value);
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
            Msg::DHCPChanged(dhcp) => {
                *self.conf.0.borrow_mut().client_conf_mut().as_ip_conf_mut() = if dhcp {
                    ipv4::ClientConfiguration::DHCP
                } else {
                    ipv4::ClientConfiguration::Fixed(Default::default())
                };

                true
            }
            Msg::SubnetChanged(value) => {
                self.fields.subnet.update(value);
                true
            }
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
            Msg::ApSSIDChanged(value) => {
                self.ap_fields.ssid.update(value);
                true
            }
            Msg::ApSSIDHiddenChanged(value) => {
                self.conf.0.borrow_mut().ap_conf_mut().ssid_hidden = value;
                true
            }
            Msg::ApAuthMethodChanged(value) => {
                self.ap_fields.auth_method.update(value.to_string());
                true
            }
            Msg::ApPasswordChanged(value) => {
                self.ap_fields.password.update(value);
                true
            }
            Msg::ApPasswordConfirmedChanged(value) => {
                self.ap_fields.password_confirmed.update(value);
                true
            }
            Msg::ApSubnetChanged(value) => {
                self.ap_fields.subnet.update(value);
                true
            }
            Msg::ApDHCPEnabledChanged(value) => {
                self.conf
                    .0
                    .borrow_mut()
                    .ap_conf_mut()
                    .as_ip_conf_mut()
                    .dhcp_enabled = value;
                true
            }
            Msg::ApDnsChanged(value) => {
                self.ap_fields.dns.update(value);
                true
            }
            Msg::ApSecondaryDnsChanged(value) => {
                self.ap_fields.secondary_dns.update(value);
                true
            }

            Msg::None => false,
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
        if self.access_points_shown {
            self.view_access_points()
        } else {
            self.view_configuration()
        }
    }
}

impl WiFi {
    fn view_access_points(&self) -> Html {
        html! {
            <>
            <MatTopAppBar>
                <MatTopAppBarNavigationIcon>
                    <span onclick=self.link.callback(move |_| Msg::ShowConfiguration(None))><MatIconButton icon="close"/></span>
                </MatTopAppBarNavigationIcon>

                <div slot="title">{"Select WiFi Network"}</div>
                <MatTopAppBarActionItems>
                    <span onclick=self.link.callback(move |_| Msg::GetAccessPoints)><MatIconButton icon="refresh"/></span>
                </MatTopAppBarActionItems>
            </MatTopAppBar>

            <CenteredGrid>
                <MatLinearProgress closed=!self.aps.is_loading()/>
                <MatList>
                {
                    for self.aps.data_ref().or(Some(&vec![])).unwrap().iter().map(|item| {
                        let ssid = item.ssid.clone();
                        let auth_method = item.auth_method;

                        let cb = self.link.callback(move |event: RequestSelectedDetail| {
                            if event.selected {
                                Msg::ShowConfiguration(Some((ssid.clone(), auth_method)))
                            } else {
                                Msg::None
                            }
                        });

                        html! {
                            <MatListItem
                                selected=false
                                tabindex=-1
                                value=item.ssid.clone()
                                graphic={GraphicType::Icon}
                                on_request_selected=cb
                                twoline=true
                            >
                                <MatIcon>{if item.auth_method == wifi::AuthMethod::None {"signal_wifi_4_bar"} else {"signal_wifi_4_bar_lock"}}</MatIcon>
                                <span>{item.ssid.clone()}</span>
                                <span slot="secondary">{strum::EnumMessage::get_message(&item.auth_method).unwrap()}</span>
                            </MatListItem>
                        }
                    })
                }
                </MatList>
            </CenteredGrid>
            </>
        }
    }

    fn view_configuration(&self) -> Html {
        let (lspan, mut mspan, rspan) = (1, 10, 1);
        let (mut ap, mut sta) = (false, false);

        match self.props.behavior {
            PluginBehavior::STA => sta = true,
            PluginBehavior::AP => ap = true,
            PluginBehavior::Mixed => {
                ap = true;
                sta = true;
                mspan = 4;
            }
        }

        html! {
            <>
                {self.props.app_bar_renderer.as_ref().unwrap().call(())}

                <Grid>
                    {self.view_configuration_cells(ap, sta, lspan, mspan, rspan)}
                </Grid>
            </>
        }
    }

    fn view_configuration_cells(
        &self,
        ap: bool,
        sta: bool,
        lspan: u32,
        mspan: u32,
        rspan: u32,
    ) -> Html {
        // TODO validity_transform={Some(MatTextField::validity_transform(|_, _| *ValidityState::new().set_bad_input(self.fields.ssid.is_valid())))}

        let aspan = lspan + mspan + rspan;

        html! {
            <>
            // Status
            <Chunk visible=ap>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    { self.status.0.borrow().ap_status_str() }
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible=sta>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    { self.status.0.borrow().client_status_str() }
                </Cell>
                <Cell span=rspan/>
            </Chunk>

            // SSID
            <Chunk visible=ap>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label="SSID"
                        disabled=!self.is_loaded()
                        value=self.ap_fields.ssid.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::ApSSIDChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.ap_fields.ssid.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible=sta>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
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
                </Cell>
                <Cell span=rspan>
                    <span slot="trailing" onclick=self.link.callback(|_| Msg::ShowAccessPoints)><MatIconButton icon="search"/></span>
                </Cell>
            </Chunk>

            // Hide SSID (AP only)
            <Chunk visible=ap>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <span>{"Hide SSID"}</span>
                    <MatSwitch
                        disabled=!self.is_loaded()
                        onchange=self.link.callback(|state| Msg::ApSSIDHiddenChanged(state))
                        checked=self.conf.0.borrow().ap_conf().map(|a| a.ssid_hidden).unwrap_or(false)
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible={sta && ap}>
                <Cell span=aspan/>
            </Chunk>

            // Authentication
            <Chunk visible=ap>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatSelect
                        outlined=true
                        label="Authentication"
                        disabled=!self.is_loaded()
                        value=self.ap_fields.auth_method.get_value_str().to_owned()
                        onselected=self.link.callback(|sd: SelectedDetail| match sd.index {
                            ListIndex::Single(Some(index)) => Msg::ApAuthMethodChanged(AuthMethod::try_from(index as u8).unwrap()),
                            _ => Msg::None,
                        })
                    >
                        {as_list(self.ap_fields.auth_method.get_value())}
                    </MatSelect>
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible=sta>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
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
                </Cell>
                <Cell span=rspan/>
            </Chunk>

            // Password
            <Chunk visible={ap && Some(wifi::AuthMethod::None) != self.ap_fields.auth_method.get_value()}>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label={
                            if Some(wifi::AuthMethod::WEP) == self.ap_fields.auth_method.get_value() {
                                "Key"
                            } else {
                                "Password"
                            }
                        }
                        disabled=!self.is_loaded()
                        value=self.ap_fields.password.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::ApPasswordChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.ap_fields.password.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible={sta && Some(wifi::AuthMethod::None) != self.fields.auth_method.get_value()}>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
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
                </Cell>
                <Cell span=rspan/>
            </Chunk>

            // Confirm password (AP only)
            <Chunk visible={ap && Some(wifi::AuthMethod::None) != self.ap_fields.auth_method.get_value()}>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label={
                            if Some(wifi::AuthMethod::WEP) == self.ap_fields.auth_method.get_value() {
                                "Confirm Key"
                            } else {
                                "Confirm Password"
                            }
                        }
                        disabled=!self.is_loaded()
                        value=self.ap_fields.password_confirmed.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::ApPasswordConfirmedChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.ap_fields.password_confirmed.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible={sta && ap && Some(wifi::AuthMethod::None) != self.ap_fields.auth_method.get_value()}>
                <Cell span=aspan/>
            </Chunk>

            // DHCP
            <Chunk visible=ap>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <span>{"DHCP Server"}</span>
                    <MatSwitch
                        disabled=!self.is_loaded()
                        onchange=self.link.callback(|state| Msg::ApDHCPEnabledChanged(state))
                        checked=self.conf.0.borrow().ap_ip_conf().map(|i| i.dhcp_enabled).unwrap_or(false)
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible=sta>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <span>{"Use DHCP"}</span>
                    <MatSwitch
                        disabled=!self.is_loaded()
                        onchange=self.link.callback(|state| Msg::DHCPChanged(state))
                        checked=self.is_dhcp()
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>

            // Subnet
            <Chunk visible=ap>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label="Subnet/Gateway"
                        disabled=!self.is_loaded()
                        value=self.ap_fields.subnet.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::ApSubnetChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.ap_fields.subnet.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible=sta>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label="Subnet/Gateway"
                        disabled=!self.is_loaded() || self.is_dhcp()
                        value=self.fields.subnet.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::SubnetChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.fields.subnet.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>

            // IP (STA only)
            <Chunk visible={sta && ap}>
                <Cell span=aspan/>
            </Chunk>
            <Chunk visible=sta>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label="IP"
                        disabled=!self.is_loaded() || self.is_dhcp()
                        value=self.fields.ip.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::IpChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.fields.ip.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>

            // DNS
            <Chunk visible=ap>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label="DNS"
                        disabled=!self.is_loaded()
                        value=self.ap_fields.dns.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::ApDnsChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.ap_fields.dns.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible=sta>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label="DNS"
                        disabled=!self.is_loaded() || self.is_dhcp()
                        value=self.fields.dns.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::DnsChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.fields.dns.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>

            // Secondary DNS
            <Chunk visible=ap>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label="Secondary DNS"
                        disabled=!self.is_loaded()
                        value=self.ap_fields.secondary_dns.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::ApSecondaryDnsChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.ap_fields.secondary_dns.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            <Chunk visible=sta>
                <Cell span=lspan/>
                <Cell span=mspan style="text-align: center;">
                    <MatTextField
                        outlined=true
                        label="Secondary DNS"
                        disabled=!self.is_loaded() || self.is_dhcp()
                        value=self.fields.secondary_dns.get_value_str().to_owned()
                        oninput=self.link.callback(|id: InputData| Msg::SecondaryDnsChanged(id.value))
                        validate_on_initial_render=true
                        auto_validate=true
                        helper={ self.fields.secondary_dns.get_error_str() }
                    />
                </Cell>
                <Cell span=rspan/>
            </Chunk>
            </>
        }
    }
}
