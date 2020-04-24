use sysinfo::{SystemExt,DiskExt};
use std::collections::HashSet;
use async_trait::async_trait;
use std::path::Path;
use super::{TimerGenerator,GenArg,Result,ExitReason};
use crate::dzen_format::DzenBuilder;
use crate::dzen_format::utils::bytes_to_ibibyte_string as byte_to_string;

pub struct DiskGen{
    sys: sysinfo::System,
    disks: Vec<String>,
    cur_disk: usize,
}

impl DiskGen {
    pub fn new() -> Self {
        DiskGen{
            sys: sysinfo::System::new(),
            disks: Vec::new(),
            cur_disk: 0,
        }
    }
}

#[async_trait]
impl TimerGenerator for DiskGen {
    async fn init(&mut self, arg: &Option<GenArg>) -> Result<()> {
        self.sys.refresh_disks_list();
        let avail_disk = self.sys.get_disks()
            .into_iter()
            .map(|d| d.get_mount_point().to_str())
            .filter(|d| d.is_some())
            .map(|d| d.unwrap())
            .collect::<HashSet<&str>>();

        if let Some(GenArg{arg: Some(a), ..}) = arg {
            for disk in a.split(",") {
                if !avail_disk.contains(disk) {
                    eprintln!("{} does not seem to be a mount point", disk);
                } else {
                    self.disks.push(disk.to_string());
                }
            }
        }

        if self.disks.is_empty() {
            eprintln!("disks list is empty for some reason");
            return Err(ExitReason::Error);
        }

        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        self.sys.refresh_disks();
        Ok(())
    }

    fn display(&self, name: &str, arg: &Option<GenArg>) -> Result<String> {
        let cur_disk = self.disks[self.cur_disk].as_str();
        let cur_path = Path::new(cur_disk);
        let disk = self.sys.get_disks()
            .into_iter()
            .find(|d| d.get_mount_point() == cur_path)
            .ok_or(ExitReason::Error)?;

        let total = disk.get_total_space();
        let avail = disk.get_available_space();
        let used = total - avail;
        let perc = ((used as f64 / total as f64) * 100.0).round();

        let o = DzenBuilder::from(cur_disk)
            .add(" ")
            .add(&perc.to_string())
            .add("%")
            .add(" ")
            .add(&byte_to_string(total))
            .name_click("1", name)
            .to_string();

        Ok(o)
    }

    async fn on_msg(&mut self, msg: String) -> Result<bool> {
        match msg.as_str() {
            "click 1" => {
                self.cur_disk += 1;
                self.cur_disk %= self.disks.len();
            },
            _ => {
                eprintln!("got unexpected message");
            }
        }
        Ok(false)
    }

    fn get_delay(&self, arg: &Option<GenArg>) -> u64 {
        arg.as_ref().and_then(|ga| ga.timeout).unwrap_or(60)
    }
}
