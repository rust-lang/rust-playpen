use std::io;
use std::io::prelude::*;
use std::mem;
use std::process::{Output, Command, Stdio, ExitStatus};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

pub struct Container {
    id: String,
}

impl Container {
    pub fn new(cmd: &str,
               args: &[String],
               env: &[(String, String)],
               name: &str) -> io::Result<Container> {
        let out = try!(run(Command::new("docker")
                                   .arg("create")
                                   .arg("--cap-drop=ALL")
                                   .arg("--memory=128m")
                                   .arg("--net=none")
                                   .arg("--pids-limit=20")
                                   .arg("--security-opt=no-new-privileges")
                                   .arg("--interactive")
                                   .args(&env.iter().map(|&(ref k, ref v)| format!("--env={}={}", k, v)).collect::<Vec<_>>())
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
        let start = Instant::now();
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
        debug!("timing: {:?}", start.elapsed());
        let mut lock = sink.lock().unwrap();
        let output = mem::replace(&mut *lock, Vec::new());
        debug!("status: {}", status);
        {
            let output_lossy = String::from_utf8_lossy(&output);
            if output_lossy.len() < 1024 {
                debug!("output: {}", output_lossy);
            } else {
                let s = output_lossy.chars().take(1024).collect::<String>();
                debug!("output (truncated): {}...", s);
            }
        }
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
    let start = Instant::now();
    let out = try!(cmd.output());
    debug!("done in: {:?}", start.elapsed());
    debug!("output: {:?}", out);
    if !out.status.success() {
        let msg = format!("process failed: {:?}\n{:?}", cmd, out);
        return Err(io::Error::new(io::ErrorKind::Other, msg))
    }
    Ok(out)
}
