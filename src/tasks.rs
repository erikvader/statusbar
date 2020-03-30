pub mod generator;
pub mod dzen;
pub mod main_task;
pub mod pipo;

#[derive(PartialEq,Eq,Clone,Copy,Debug)]
pub enum ExitReason {
    Signal,
    Error,
    SignalError,
    Normal
}

impl ExitReason {
    pub fn combine(self, other: Self) -> Self {
        fn rec(me: ExitReason, other: ExitReason, second: bool) -> ExitReason {
            match (me, other) {
                (ExitReason::Error, ExitReason::Signal) => ExitReason::SignalError,
                (ExitReason::SignalError, _) => ExitReason::SignalError,
                (ExitReason::Normal, a) => a,
                (a, b) if a == b => a,
                (_, _) if second => panic!("there is some missed case"),
                (a, b) => rec(b, a, true)
            }
        }
        rec(self, other, false)
    }
}