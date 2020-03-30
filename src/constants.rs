use crate::tasks::generator::GenId;

pub const FIFO_PATH: &str = "/tmp/statusbar_fifo";
pub const MPSC_SIZE: usize = 32;

pub type Msg = (GenId, String);

