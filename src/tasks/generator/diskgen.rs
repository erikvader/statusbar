use sysinfo::{SystemExt,DiskExt};
use async_trait::async_trait;
use std::path::PathBuf;
use super::{TimerGenerator,GenArg,Result,ExitReason};

const LEVELS: &[(i32, &str)] = &[(90, "yellow"), (95, "red")];
const FS_WHITELIST: &[&str] = &["nfs", "ext4"];

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
        if let Some(a) = &arg.arg {
            for disk in a.split(",") {
                self.disks.push(PathBuf::from(disk));
            }
        }

        if self.disks.is_empty() {
            log::warn!("disks list is empty for some reason");
            return Err(ExitReason::NonFatal);
        }

        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        self.sys.refresh_disks_list();
        self.sys.refresh_disks();
        Ok(())
    }

    fn display(&self, _name: &str, arg: &GenArg) -> Result<String> {
        let mut bu = arg.get_builder().new_section();
        let mut missing = self.disks.len();

        for disk in self.sys.get_disks() {

            let correct_fs = std::str::from_utf8(disk.get_file_system())
                .map_or(false, |fs| FS_WHITELIST.contains(&fs));

            let our_disk = self.disks.iter().any(|p| p == disk.get_mount_point());

            if !correct_fs || !our_disk {
                continue;
            }

            missing -= 1;

            let total = disk.get_total_space();
            let avail = disk.get_available_space();
            let used = total - avail;
            let perc = ((used as f64 / total as f64) * 100.0).round() as i32;

            bu = bu.add_not_empty("/")
                .new_section()
                .add(perc.to_string())
                .color_step(perc, LEVELS);
        }

        for _ in 0..missing {
            bu = bu.add_not_empty("/")
                .new_section()
                .add("xx");
        }

        Ok(bu.to_string())
    }

    fn get_delay(&self, arg: &GenArg) -> u64 {
        arg.timeout.unwrap_or(60)
    }
}
