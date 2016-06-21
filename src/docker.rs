use std::io;
use std::io::prelude::*;
use std::mem;
use std::process::{Output, Command, Stdio, ExitStatus};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use wait_timeout::ChildExt;

pub struct Container {
    id: String,
}

impl Container {
    pub fn new(cmd: &str,
               args: &[String],
               name: &str) -> io::Result<Container> {
        let out = try!(run(Command::new("docker")
                                   .arg("create")
                                   .arg("--cap-drop=ALL")
                                   .arg("--memory=128m")
                                   .arg("--net=none")
                                   .arg("--pids-limit=5")
                                   .arg("--security-opt=no-new-privileges")
                                   .arg("--interactive")
                                   .arg(name)
                                   .arg(cmd)
                                   .stderr(Stdio::inherit())
                                   .args(args)));
        let stdout = String::from_utf8_lossy(&out.stdout);
        Ok(Container {
            id: stdout.trim().to_string(),
        })
    }

    pub fn run(&self,
               input: &[u8],
               timeout: Duration)
               -> io::Result<(ExitStatus, Vec<u8>, bool)> {
        let mut cmd = Command::new("docker");
        cmd.arg("start")
           .arg("--attach")
           .arg("--interactive")
           .arg(&self.id)
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        debug!("attaching with {:?}", cmd);
        let mut cmd = try!(cmd.spawn());
        try!(cmd.stdin.take().unwrap().write_all(input));
        debug!("input written, now waiting");

        let mut stdout = cmd.stdout.take().unwrap();
        let mut stderr = cmd.stderr.take().unwrap();
        let sink = Arc::new(Mutex::new(Vec::new()));
        let sink2 = sink.clone();
        let stdout = thread::spawn(move || append(&sink2, &mut stdout));
        let sink2 = sink.clone();
        let stderr = thread::spawn(move || append(&sink2, &mut stderr));

        let (status, timeout) = match try!(cmd.wait_timeout(timeout)) {
            Some(status) => {
                debug!("finished before timeout");
                // TODO: document this
                (unsafe { mem::transmute(status) }, false)
            }
            None => {
                debug!("timeout, going to kill");
                try!(run(Command::new("docker").arg("kill").arg(&self.id)));
                (try!(cmd.wait()), true)
            }
        };
        stdout.join().unwrap();
        stderr.join().unwrap();
        let mut lock = sink.lock().unwrap();
        let output = mem::replace(&mut *lock, Vec::new());
        debug!("status: {}", status);
        debug!("output: {}", String::from_utf8_lossy(&output));
        Ok((status, output, timeout))
    }
}

fn append(into: &Mutex<Vec<u8>>, from: &mut Read) {
    let mut buf = [0; 1024];
    while let Ok(amt) = from.read(&mut buf) {
        if amt == 0 {
            break
        }
        into.lock().unwrap().extend_from_slice(&buf[..amt]);
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        run(Command::new("docker")
                    .arg("rm")
                    .arg("--force")
                    .arg(&self.id)).unwrap();
    }
}

fn run(cmd: &mut Command) -> io::Result<Output> {
    debug!("spawning: {:?}", cmd);
    let out = try!(cmd.output());
    debug!("output: {:?}", out);
    if !out.status.success() {
        let msg = format!("process failed: {:?}\n{:?}", cmd, out);
        return Err(io::Error::new(io::ErrorKind::Other, msg))
    }
    Ok(out)
}
