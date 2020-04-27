use sysinfo::{SystemExt,DiskExt};
use std::collections::HashSet;
use async_trait::async_trait;
use std::path::PathBuf;
use super::{TimerGenerator,GenArg,Result,ExitReason};

const LEVELS: &[(i32, &str)] = &[(90, "yellow"), (95, "red")];

pub struct DiskGen{
    sys: sysinfo::System,
    disks: Vec<PathBuf>,
}

impl DiskGen {
    pub fn new() -> Self {
        DiskGen{
            sys: sysinfo::System::new(),
            disks: Vec::new(),
        }
    }
}

#[async_trait]
impl TimerGenerator for DiskGen {
    async fn init(&mut self, arg: &GenArg) -> Result<()> {
        self.sys.refresh_disks_list();
        let avail_disk = self.sys.get_disks()
            .into_iter()
            .map(|d| d.get_mount_point().to_str())
            .filter(|d| d.is_some())
            .map(|d| d.unwrap())
            .collect::<HashSet<&str>>();

        if let Some(a) = &arg.arg {
            for disk in a.split(",") {
                if !avail_disk.contains(disk) {
                    eprintln!("{} does not seem to be a mount point", disk);
                } else {
                    self.disks.push(PathBuf::from(disk));
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

    fn display(&self, _name: &str, arg: &GenArg) -> Result<String> {
        let mut bu = arg.get_builder().new_section();
        for disk in self.sys.get_disks() {
            if !self.disks.iter().any(|p| p == disk.get_mount_point()) {
                continue;
            }

            let total = disk.get_total_space();
            let avail = disk.get_available_space();
            let used = total - avail;
            let perc = ((used as f64 / total as f64) * 100.0).round() as i32;

            bu = bu.add_not_empty("/")
                .new_section()
                .add(perc.to_string())
                .add("%")
                .color_step(perc, LEVELS);
        }

        Ok(bu.to_string())
    }

    fn get_delay(&self, arg: &GenArg) -> u64 {
        arg.timeout.unwrap_or(60)
    }
}
