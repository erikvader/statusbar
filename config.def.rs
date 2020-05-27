use super::bar;
use super::bar::SetupBuilder as SB;
use super::bar::BarBuilder as BB;
use super::bar::GenBuilder as GB;
use super::tasks::generator::GenType as GT;
use super::dzen_format::DzenBuilder as DB;

pub const FIFO_PATH:   &str = "/tmp/statusbar_fifo";
pub const DZEN_FONT:   &str = "Bitstream Vera Sans:pixelsize=14:antialias=true:hinting=true";
pub const ICON_PATH:   &str = "~/Documents/statusbar/icons";
pub const SCRIPT_PATH: &str = "~/Documents/statusbar/scripts";

pub fn theme<S>(c: S) -> Option<&'static str>
where S: AsRef<str>
{
    match c.as_ref() {
        "fg"         => Some("#dfdfdf"),
        "bg"         => Some("#333333"),
        "lightbg"    => Some("#505050"),
        "urgent"     => Some("#bd2c40"),
        "hotpink"    => Some("#ff69b4"),
        "orange"     => Some("#ffb52a"),
        "yellow2"    => Some("#eeee00"),
        "blue2"      => Some("#00ace6"),
        "darkorange" => Some("#ff8c00"),
        "magenta"    => Some("#ff00ff"),
        _            => None,
    }
}

pub fn icon_theme<S>(c: S) -> Option<&'static str>
where S: AsRef<str>
{
    match c.as_ref() {
        "battery"      => Some("kanna"),
        "volume"       => Some("sonico"),
        "temperature"  => Some("salamander"),
        "temperature2" => Some("yoko"),
        "cpu"          => Some("balzac"),
        "ram"          => Some("ram"),
        "time"         => Some("lucy"),
        "wifi"         => Some("vert"),
        "netspeed"     => Some("rem"),
        "disks"        => Some("hinata"),
        _              => None,
    }
}

fn pre_icon(i: &'static str) -> DB<'static> {
    DB::new().append_icon(i).rpad(3)
}

pub fn config() -> bar::Result {
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
        .build()
}
