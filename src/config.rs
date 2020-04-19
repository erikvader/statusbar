use super::bar;
use super::bar::SetupBuilder as SB;
use super::bar::BarBuilder as BB;
use super::bar::GenBuilder as GB;
use super::tasks::generator::GenType as GT;
use super::dzen_format::DzenBuilder as DB;

pub const FIFO_PATH:   &str = "/tmp/statusbar_fifo";
pub const DZEN_FONT:   &str = "xft:Ubuntu Mono:pixelsize=14:antialias=true:hinting=true";
pub const ICON_PATH:   &str = "~/Documents/statusbar/icons";
pub const SCRIPT_PATH: &str = "~/Documents/statusbar/scripts";

pub fn config() -> bar::Result {
    SB::new()
        .add_bar(BB::new("DisplayPort-0")
                 .add_left(GB::new(GT::CPU))
                 .add_left(GB::new(GT::RAM))
                 .add_right(GB::new(GT::FOL)
                            .argument("pulseaudio.py"))
                 .add_right(GB::new(GT::IP)
                            .argument("enp4s0"))
                 .add_right(GB::new(GT::TEMP)
                            .argument("Package id 0"))
                 .add_right(GB::new(GT::TEMP)
                            .argument("edge"))
                 .add_right(GB::new(GT::NET)
                            .argument("enp4s0")
                            .timeout(1))
                 .add_right(GB::new(GT::DISK)
                            .argument("/,/home,/media/3TB,/media/4TB"))
                 .add_right(GB::new(GT::CPU))
                 .add_right(GB::new(GT::TIME))
                 .tray(true))
        .map_other(|output| BB::new(output)
                   .add_right(GB::new(GT::TIME)))
        .separator(DB::from(" | ").colorize("red").to_string())
        .build()
}
