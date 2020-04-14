use async_trait::async_trait;
use super::{GenArg,ExitReason,Msg,GenId,Generator};
use crate::dzen_format::DzenBuilder;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::stream::StreamExt;
use dbus_tokio::connection;
use dbus::nonblock as DN;
use std::time::Duration;
use core::ops::Deref;
use std::sync::Arc;
use dbus::strings::Path;
use std::collections::HashMap;
use simple_error::SimpleError;
use dbus::arg::RefArg;

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

pub struct IpGen;

impl IpGen {
    pub fn new() -> Self {
        IpGen
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
    show_ssid: &bool,
    name: &str,
    conn: Arc<C>
) -> String
where C: DN::NonblockReply
{
    let mut bu = DzenBuilder::new();
    bu = bu.add(match state {
        70 => "C",
        60 => "L",
        _ => "NC"
    }).add(" ");

    let to_show;
    if let Ok(devpath) = get_device(interface, conn.clone()).await {
        let show_ssid = *show_ssid && {
            match is_device_wifi(&devpath, conn.clone()).await {
                Ok(w) => w,
                Err(_) => false
            }
        };

        if show_ssid {
            if let Ok(ssid) = get_device_ssid(&devpath, conn.clone()).await {
                to_show = ssid;
            } else {
                to_show = "no ssid".to_string();
            }
        } else {
            if let Ok(i) = get_device_ip(&devpath, conn).await {
                to_show = i;
            } else {
                to_show = "no ip".to_string();
            }
        }
    } else {
        to_show = "no device".to_string();
    }

    bu.add(&to_show).name_click("1", name).to_string()
}

#[async_trait]
impl Generator for IpGen {
    async fn start(
        &mut self,
        to_printer: broadcast::Sender<Msg>,
        mut from_pipo: mpsc::Receiver<String>,
        id: GenId,
        arg: Option<GenArg>,
        name: String
    ) -> ExitReason
    {
        // find interface from argument
        let interface =
            if let Some(GenArg{arg: Some(iface), ..}) = arg {
                iface
            } else {
                eprintln!("I want an interface as argument");
                return ExitReason::Error;
            };

        // connect to dbus
        let (mut resource, conn) = unwrap_er!(connection::new_system_sync()
                                          .map_err(|e| {
                                              eprintln!("dbus: {}", e);
                                              ExitReason::Error
                                          }));

        // declare main loop
        let main_loop = async {
            let sig = dbus::message::MatchRule::new_signal(ROOT_IF, "StateChanged");
            let (mm, mut stream) = conn.add_match(sig).await?.stream();

            // TODO: use a state that indicates that networkmanager is
            // enabled/active if this fails.
            let mut state: u32 = get_network_state(conn.clone()).await?;
            let mut show_ssid = true;

            loop {
                let s = get_string(&interface, state, &show_ssid, &name, conn.clone()).await;
                if let Err(_) = to_printer.send((id, s)) {
                    return Err(ExitReason::Error);
                }

                tokio::select! {
                    msg = from_pipo.recv() => match msg {
                        None => break,
                        Some(s) if s == "click 1" => show_ssid = !show_ssid,
                        _ => ()
                    },
                    Some((_, (s,))) = stream.next() => {
                        state = s;
                    }
                }
            };

            conn.remove_match(mm.token()).await?;

            Ok(())
        };

        // wait for main loop or dbus disconnect
        let ret = tokio::select! {
            err = &mut resource => {
                eprintln!("dbus connection lost. '{}'", err);
                Err(ExitReason::Error)
            },
            ret = main_loop => ret
        };

        match ret {
            Ok(_)   => ExitReason::Normal,
            Err(er) => er
        }
    }
}
