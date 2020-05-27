use nix::sys::signal;
use nix::unistd::Pid;
use tokio::process::Child;
use std::process;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context,Poll};

fn terminate(c: &Child) -> Result<(), nix::Error> {
    let id = Pid::from_raw(c.id() as i32);
    signal::kill(id, signal::Signal::SIGTERM)
}

pub struct ChildTerminator {
    inner: Child,
    kill_on_drop: bool
}

impl ChildTerminator {
    pub fn new(c: Child) -> Self {
        Self {
            inner: c,
            kill_on_drop: true
        }
    }

    pub fn terminate(&mut self) -> Result<(), nix::Error> {
        self.kill_on_drop = false;
        Ok(terminate(&self.inner)?)
    }

    pub fn as_mut_ref(&mut self) -> &mut Child {
        &mut self.inner
    }

    pub fn as_ref(&self) -> &Child {
        &self.inner
    }
}

impl Future for ChildTerminator {
    type Output = tokio::io::Result<process::ExitStatus>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let ret = Pin::new(&mut self.inner).poll(cx);

        if let Poll::Ready(Ok(_)) = ret {
            self.kill_on_drop = false;
        }

        ret
    }
}

// Doesn't reap child. But if this is called then is the process
// terminating probably, so it's fine
impl std::ops::Drop for ChildTerminator {
    fn drop(&mut self) {
        if !self.kill_on_drop {
            return
        }

        let pid = self.as_ref().id();
        log::debug!("sending SIGTERM to {}", pid);
        if let Err(e) = self.terminate() {
            log::warn!("couldn't kill {} because '{}'", pid, e);
        }
    }
}
