pub mod generator;
pub mod dzen;
pub mod main_task;
pub mod pipo;

use crate::tasks::generator::GenId;

#[derive(Clone,Debug)]
pub enum Msg {
    Gen(GenId, String),
    Tray,
}

#[derive(PartialEq,Eq,Clone,Copy,Debug)]
pub enum ExitReason {
    Signal,
    Error,
    Normal,
    NonFatal,
}

#[derive(Debug)]
pub enum ProcessExitReason {
    Okay,
    Error(Vec<ExitReason>),
}

type PRE = ProcessExitReason;
type ER = ExitReason;

impl ProcessExitReason {
    pub fn new() -> Self {
        ProcessExitReason::Okay
    }

    pub fn combine(self, reason: ExitReason) -> Self {
        match (self, reason) {
            (p, ER::Normal) => p,
            (p, ER::NonFatal) => p,
            (PRE::Okay, e) => PRE::Error(vec![e]),
            (PRE::Error(mut v), e) if !v.contains(&e) => {
                v.push(e);
                PRE::Error(v)
            },
            (p, _) => p,
        }
    }

    pub fn get_exit_code(&self) -> i32 {
        match self {
            PRE::Okay => 0,
            PRE::Error(v) if v.contains(&ER::Error) && v.contains(&ER::Signal) => 3,
            PRE::Error(v) if v.contains(&ER::Signal) => 2,
            PRE::Error(v) if v.contains(&ER::Error) => 1,
            PRE::Error(_) => panic!("invalid state")
        }
    }
}

impl<E> From<E> for ExitReason
where E: std::fmt::Display
{
    fn from(e: E) -> Self {
        // TODO: don't just print
        log::error!("got error '{}', converted it to ExitReason::Error", e);
        ExitReason::Error
    }
}
