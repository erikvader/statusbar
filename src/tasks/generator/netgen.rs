use sysinfo::{SystemExt,NetworkExt};
use std::collections::HashSet;
use async_trait::async_trait;
use super::{TimerGenerator,GenArg,Result,ExitReason};
use crate::dzen_format::DzenBuilder;
use crate::dzen_format::utils::bytes_to_ibibyte_string as byte_to_string;

pub struct NetGen{
    sys: sysinfo::System,
    interfaces: Vec<String>,
    cur_if: usize,
    total: bool,
    timeout: u64
}

impl NetGen {
    pub fn new() -> Self {
        NetGen{
            sys: sysinfo::System::new(),
            interfaces: Vec::new(),
            cur_if: 0,
            total: false,
            timeout: 0
        }
    }
}

#[async_trait]
impl TimerGenerator for NetGen {
    async fn init(&mut self, arg: &GenArg) -> Result<()> {
        self.sys.refresh_networks_list();
        let avail_net = self.sys.get_networks()
            .into_iter()
            .map(|(x, _)| x.as_str())
            .collect::<HashSet<&str>>();

        if let Some(a) = &arg.arg {
            for iface in a.split(" ") {
                if !avail_net.contains(iface) {
                    eprintln!("{} is not a connected interface", iface);
                    return Err(ExitReason::Error);
                }
                self.interfaces.push(iface.to_string());
            }
        }

        if self.interfaces.is_empty() {
            eprintln!("interfaces list is for some reason empty");
            return Err(ExitReason::Error);
        }

        self.timeout = self.get_delay(arg);

        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        self.sys.refresh_networks();
        Ok(())
    }

    fn display(&self, name: &str, arg: &GenArg) -> Result<String> {
        let cur_if = self.interfaces[self.cur_if].as_str();
        let net = self.sys.get_networks()
            .into_iter()
            .find(|(iface, _)| *iface == cur_if)
            .map(|(_, n)| n)
            .ok_or(ExitReason::Error)?;

        let (up, down) = if !self.total {
            (net.get_transmitted() / self.timeout, net.get_received() / self.timeout)
        } else {
            (net.get_total_transmitted(), net.get_total_received())
        };

        let o = arg.get_builder()
            .add(byte_to_string(up))
            .maybe_add(!self.total, "/s")
            .add(" / ")
            .add(byte_to_string(down))
            .maybe_add(!self.total, "/s")
            .name_click(1, name)
            .name_click(3, name)
            .to_string();

        Ok(o)
    }

    async fn on_msg(&mut self, msg: String) -> Result<bool> {
        match msg.as_str() {
            "click 3" => {
                self.cur_if += 1;
                self.cur_if %= self.interfaces.len();
            },
            "click 1" => {
                self.total = !self.total;
            },
            _ => {
                eprintln!("got unexpected message");
            }
        }
        Ok(false)
    }

    fn get_delay(&self, arg: &GenArg) -> u64 {
        arg.timeout.unwrap_or(2)
    }
}
