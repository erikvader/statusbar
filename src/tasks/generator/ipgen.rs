use async_trait::async_trait;
use super::{GenArg,ExitReason,DBusGenerator};
use super::Result as EResult;
use crate::dzen_format::DzenBuilder;
use dbus_tokio::connection;
use dbus::nonblock as DN;
use std::time::Duration;
use core::ops::Deref;
use std::sync::Arc;
use dbus::strings::Path;
use std::collections::HashMap;
use simple_error::SimpleError;
use dbus::arg::RefArg;
use dbus_tokio::connection::IOResource;

// https://developer.gnome.org/NetworkManager/stable/index.html

const BUSNAME: &str = "org.freedesktop.NetworkManager";
const ROOT_OBJ: &str =  "/org/freedesktop/NetworkManager";
const ROOT_IF: &str = BUSNAME;
const PROP_IF: &str = "org.freedesktop.DBus.Properties";
const DEVICE_IF: &str = "org.freedesktop.NetworkManager.Device";
const IP4_IF: &str = "org.freedesktop.NetworkManager.IP4Config";
const WIFI_IF: &str = "org.freedesktop.NetworkManager.Device.Wireless";
const AP_IF: &str = "org.freedesktop.NetworkManager.AccessPoint";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct IpGen {
    show_ssid: bool,
    state: u32,
    interface: String
}

impl IpGen {
    pub fn new() -> Self {
        IpGen{
            show_ssid: true,
            state: 0,
            interface: "".to_string()
        }
    }
}

async fn get_device<C,T>(interface: &str, conn: C) -> Result<Path<'_>>
where C: Deref<Target=T>,
      T: DN::NonblockReply
{
    let root_proxy = DN::Proxy::new(BUSNAME, ROOT_OBJ, Duration::from_secs(5), conn);
    let (device,): (dbus::strings::Path,) = root_proxy.method_call(ROOT_IF, "GetDeviceByIpIface", (interface,)).await?;
    Ok(device)
}

async fn is_device_wifi<C,T>(path: &Path<'_>, conn: C) -> Result<bool>
where C: Deref<Target=T>,
      T: DN::NonblockReply
{
    let devp = DN::Proxy::new(BUSNAME, path, Duration::from_secs(5), conn);
    let (devtype,): (dbus::arg::Variant<u32>,) = devp.method_call(PROP_IF, "Get", (DEVICE_IF, "DeviceType",)).await?;
    // 2 = wifi
    // 1 = wired
    // 0 = unknown
    // n = something else
    Ok(devtype.0 == 2)
}

async fn get_device_ip<C>(path: &Path<'_>, conn: Arc<C>) -> Result<String>
where C: DN::NonblockReply
{
    let devp = DN::Proxy::new(BUSNAME, path, Duration::from_secs(5), conn.clone());
    let (devstate,): (dbus::arg::Variant<u32>,) = devp.method_call(PROP_IF, "Get", (DEVICE_IF, "State",)).await?;
    // 100 = activated and Ip4Config is valid to call
    if devstate.0 != 100 {
        return Err(Box::new(SimpleError::new("interface not connected it seems")));
    }

    let (ip_path,): (dbus::arg::Variant<Path>,) = devp.method_call(PROP_IF, "Get", (DEVICE_IF, "Ip4Config",)).await?;
    let ip_prox = DN::Proxy::new(BUSNAME, ip_path.0, Duration::from_secs(5), conn);

    let (ip4,): (dbus::arg::Variant<Vec<HashMap<String, dbus::arg::Variant<Box<dyn dbus::arg::RefArg>>>>>,)
        = ip_prox.method_call(PROP_IF, "Get", (IP4_IF, "AddressData",)).await?;

    if ip4.0.is_empty() {
        return Err(Box::new(SimpleError::new("no IP assigned")));
    }

    let adr = ip4.0[0].get("address").expect("this should exist").as_str().expect("this should be a string");

    Ok(adr.to_string())
}

async fn get_device_ssid<C>(path: &Path<'_>, conn: Arc<C>) -> Result<String>
where C: DN::NonblockReply
{
    let devp = DN::Proxy::new(BUSNAME, path, Duration::from_secs(5), conn.clone());
    let (ap_prox,): (dbus::arg::Variant<Path>,) = devp.method_call(PROP_IF, "Get", (WIFI_IF, "ActiveAccessPoint",)).await?;

    let app = DN::Proxy::new(BUSNAME, ap_prox.0, Duration::from_secs(5), conn);
    let (ssid,): (dbus::arg::Variant<Vec<u8>>,) = app.method_call(PROP_IF, "Get", (AP_IF, "Ssid",)).await?;
    Ok(String::from_utf8(ssid.0).unwrap_or("decode error".to_string()))
}

async fn get_network_state<C>(conn: Arc<C>) -> Result<u32>
where C: DN::NonblockReply
{
    let root_proxy = DN::Proxy::new(BUSNAME, ROOT_OBJ, Duration::from_secs(5), conn);
    let (state,): (u32,) = root_proxy.method_call(ROOT_IF, "state", ()).await?;
    // 70 = full
    // 60 = limited
    // 50 = no
    // 40 = connecting
    // 30 = disconnecting
    // 20 = disconnected
    // 10 = not enabled
    // 0  = unknown
    Ok(state)
}

async fn get_string<C>(
    interface: &str,
    state: u32,
    show_ssid: bool,
    name: &str,
    conn: Arc<C>,
    arg: &GenArg
) -> String
where C: DN::NonblockReply
{
    let mut bu = arg.get_builder();
    // bu = bu.add(match state {
    //     // 70 => "C",
    //     60 => "L",
    //     _ => "NC"
    // }).add(" ");

    let to_show;
    match get_device(interface, conn.clone()).await {
        Err(e) => {
            to_show = "no device".to_string();
            eprintln!("no device cuz {}", e);
        }
        Ok(devpath) => {
            let show_ssid = show_ssid && {
                match is_device_wifi(&devpath, conn.clone()).await {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("not wifi cuz {}", e);
                        false
                    }
                }
            };

            if show_ssid {
                match get_device_ssid(&devpath, conn.clone()).await {
                    Err(e) => {
                        to_show = "no ssid".to_string();
                        eprintln!("no ssid cuz {}", e);
                    }
                    Ok(ssid) => {
                        to_show = ssid;
                    }
                }
            } else {
                match get_device_ip(&devpath, conn).await {
                    Err(e) => {
                        to_show = "no ip".to_string();
                        eprintln!("no ip cuz {}", e);
                    }
                    Ok(i) => {
                        to_show = i;
                    }
                }
            }
        }
    }

    if state < 60 {
        bu = bu.add("not connected").colorize("gray");
    } else {
        bu = bu.add_trunc(10, to_show).name_click(1, name);
        if state == 60 {
            bu = bu.colorize("yellow");
        }
    }

    bu.to_string()
}

#[async_trait]
impl DBusGenerator for IpGen {
    fn get_connection(&self) -> EResult<(IOResource<DN::SyncConnection>, Arc<DN::SyncConnection>)> {
        Ok(connection::new_system_sync()?)
    }

    async fn init(&mut self, arg: &GenArg, conn: Arc<DN::SyncConnection>) -> EResult<()> {
        // find interface from argument
        self.interface =
            if let Some(iface) = &arg.arg {
                iface.to_string()
            } else {
                eprintln!("I want an interface as argument");
                return Err(ExitReason::Error);
            };

        // TODO: use a state that indicates that networkmanager is
        // enabled/active if this fails.
        self.state = get_network_state(conn).await?;

        Ok(())
    }

    async fn update(&mut self, conn: Arc<DN::SyncConnection>, name: &str, arg: &GenArg) -> EResult<String> {
        Ok(get_string(&self.interface, self.state, self.show_ssid, name, conn.clone(), arg).await)
    }

    fn interesting_signals(&self) -> Vec<dbus::message::MatchRule<'static>> {
        let sig = dbus::message::MatchRule::new_signal(ROOT_IF, "StateChanged");
        vec!(sig)
    }

    async fn handle_signal(&mut self, _sig: usize, data: dbus::message::Message) -> EResult<()> {
        if let Some(s) = data.get1() {
            self.state = s;
        } else {
            eprintln!("signal didn't contain the new state");
        }
        Ok(())
    }

    async fn handle_msg(&mut self, msg: String) -> EResult<()> {
        if msg == "click 1" {
            self.show_ssid = !self.show_ssid;
        }
        Ok(())
    }
}
