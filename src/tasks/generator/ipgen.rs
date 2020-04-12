use async_trait::async_trait;
use super::{TimerGenerator,GenArg,Result,ExitReason};
use crate::dzen_format::DzenBuilder;
use nix::sys::socket::{SockAddr,IpAddr};

// https://docs.rs/dbus/0.8.2/dbus/
// https://docs.rs/dbus-tokio/0.5.1/dbus_tokio/
// https://developer.gnome.org/NetworkManager/1.2/spec.html

pub struct IpGen {
    addr: String,
    iface: String
}

impl IpGen {
    pub fn new() -> Self {
        IpGen{
            addr: "".to_string(),
            iface: "".to_string()
        }
    }
}

#[async_trait]
impl TimerGenerator for IpGen {
    async fn init(&mut self, arg: &Option<GenArg>) -> Result<()> {
        if let Some(GenArg{arg: Some(a), ..}) = arg {
            self.iface = a.to_string();
        } else {
            eprintln!("I need an interface");
            return Err(ExitReason::Error);
        };

        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        for ifaddr in nix::ifaddrs::getifaddrs()? {
            if ifaddr.interface_name == self.iface {
                if let Some(SockAddr::Inet(sock)) = ifaddr.address {
                    if let IpAddr::V4(a) = sock.ip() {
                        self.addr = a.to_string();
                        return Ok(());
                    }
                }
            }
        }
        self.addr = "not connected".to_string();
        Ok(())
    }

    fn display(&self, _name: &str) -> Result<String> {
        Ok(self.addr.to_string())
    }
}
