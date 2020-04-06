use super::bar;
use super::bar::SetupBuilder as SB;
use super::bar::BarBuilder as BB;
use super::bar::GenBuilder as GB;
use super::tasks::generator::GenType as GT;

pub const FIFO_PATH: &str = "/tmp/statusbar_fifo";
pub const DZEN_FONT: &str = "xft:Ubuntu Mono:pixelsize=14:antialias=true:hinting=true";

pub fn config() -> bar::Result {
    SB::new()
        .add_bar(BB::new("DisplayPort-0")
                 .add_left(GB::new(GT::CPU))
                 .add_left(GB::new(GT::RAM))
                 .add_right(GB::new(GT::CPU))
                 .add_right(GB::new(GT::RAM))
                 .tray(true))
        .build()
}
