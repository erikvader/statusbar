use nix::sys::signal;
use nix::unistd::Pid;
use tokio::process::Child;

pub fn terminate(c: &Child) -> Result<(), nix::Error> {
    let id = Pid::from_raw(c.id() as i32);
    signal::kill(id, signal::Signal::SIGTERM)
}

pub async fn terminate_wait(c: &mut Child) -> Result<std::process::ExitStatus, Box<dyn std::error::Error>> {
    if let Err(e) = terminate(c) {
        log::warn!("couldn't kill: {}", e);
        return Err(Box::new(e));
    }

    match c.await {
        Err(e) => {
            log::warn!("couldn't await: {}", e);
            return Err(Box::new(e));
        },
        Ok(e) => Ok(e)
    }
}
