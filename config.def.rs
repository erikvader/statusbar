use super::bar::SetupBuilder as SB;
use super::bar::BarBuilder as BB;
use super::bar::GenBuilder as GB;
use super::tasks::generator::GenType as GT;
use super::dzen_format::DzenBuilder as DB;
use super::dzen_format::config::Config;

pub const FIFO_PATH:   &str = "/tmp/statusbar_fifo";
pub const DZEN_FONT:   &str = "Bitstream Vera Sans:pixelsize=14:antialias=true:hinting=true";
pub const ICON_PATH:   &str = "~/Documents/statusbar/icons";
pub const SCRIPT_PATH: &str = "~/Documents/statusbar/scripts";

lazy_static::lazy_static! {
    pub static ref THEME: Config<'static> = {
        let mut h = Config::new();
        h.color.insert("fg",         "#dfdfdf");
        h.color.insert("bg",         "#333333");
        h.color.insert("lightbg",    "#505050");
        h.color.insert("urgent",     "#bd2c40");
        h.color.insert("hotpink",    "#ff69b4");
        h.color.insert("orange",     "#ffb52a");
        h.color.insert("yellow2",    "#eeee00");
        h.color.insert("blue2",      "#00ace6");
        h.color.insert("darkorange", "#ff8c00");
        h.color.insert("magenta",    "#ff00ff");

        h.icon.insert("battery",     "kanna");
        h.icon.insert("volume",      "sonico");
        h.icon.insert("temperature", "salamander");
        h.icon.insert("cpu",         "balzac");
        h.icon.insert("ram",         "ram");
        h.icon.insert("time",        "lucy");
        h.icon.insert("wifi",        "vert");
        h.icon.insert("netspeed",    "rem");
        h.icon.insert("disk",        "miku");

        h
    };
}

fn pre_icon(i: &'static str) -> DB<'static> {
    DB::new()
        .use_theme(&THEME)
        .append_icon(i)
        .rpad(3)
}

pub fn config() -> SB {
    SB::new()
        .add_bar(BB::new("DisplayPort-0")
                 .add_left(GB::new(GT::ECHO)
                           .name("xmonad_DisplayPort-0"))
                 .add_right(GB::new(GT::ONE)
                            .argument("pacman.sh"))
                 .add_right(GB::new(GT::ONE)
                            .argument("statusbar_progmode")
                            .name("progmode"))
                 .add_right(GB::new(GT::ONE)
                            .argument("pulseaudio.py")
                            .prepend(pre_icon("volume")))
                 .add_right(GB::new(GT::DISK)
                            .argument("/,/media/data"))
                 .add_right(GB::new(GT::NET)
                            .argument("enp4s0")
                            .prepend(pre_icon("netspeed")))
                 .add_right(GB::new(GT::TEMP)
                            .argument("Package id 0")
                            .prepend(pre_icon("temperature")))
                 .add_right(GB::new(GT::RAM)
                            .prepend(pre_icon("ram")))
                 .add_right(GB::new(GT::CPU)
                            .prepend(pre_icon("cpu")))
                 .add_right(GB::new(GT::IP)
                            .argument("enp4s0")
                            .prepend(pre_icon("wifi")))
                 .add_right(GB::new(GT::BAT)
                            .prepend(pre_icon("battery")))
                 .add_right(GB::new(GT::TIME)
                            .prepend(pre_icon("time")))
                 .tray(true))
        .map_other(|output| BB::new(&output)
                   .add_left(GB::new(GT::ECHO)
                             .name(String::from("xmonad_") + &output))
                   .add_right(GB::new(GT::TIME)
                              .prepend(pre_icon("time"))))
        .separator(DB::new()
                   .lpad(5)
                   .rect(2, 20)
                   .rpad(5)
                   .colorize("white")
                   .to_string())
}
