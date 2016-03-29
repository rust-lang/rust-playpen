#![feature(process_exec)]

extern crate libc;
extern crate lru_cache;

use lru_cache::LruCache;

use std::cell::RefCell;
use std::fmt;
use std::error::Error;
use std::io::{self, Write};
use std::str::FromStr;
use std::process::{Command, ExitStatus, Stdio};
use std::os::unix::process::CommandExt;

/// Error type holding a description
pub struct StringError(pub String);

impl Error for StringError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum ReleaseChannel {
    Stable = 0,
    Beta = 1,
    Nightly = 2,
}

impl FromStr for ReleaseChannel {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stable" => Ok(ReleaseChannel::Stable),
            "beta" => Ok(ReleaseChannel::Beta),
            "nightly" => Ok(ReleaseChannel::Nightly),
            _ => Err(StringError(format!("unknown release channel {}", s))),
        }
    }
}

/// Helper method for safely invoking a command inside a playpen
pub fn exec(channel: ReleaseChannel,
            cmd: &'static str,
            args: Vec<String>,
            input: String)
            -> io::Result<(ExitStatus, Vec<u8>)> {
    #[derive(PartialEq, Eq, Hash)]
    struct CacheKey {
        channel: ReleaseChannel,
        cmd: &'static str,
        args: Vec<String>,
        input: String,
    }

    thread_local! {
        static CACHE: RefCell<LruCache<CacheKey, (ExitStatus, Vec<u8>)>> =
            RefCell::new(LruCache::new(256))
    }

    // Build key to look up
    let key = CacheKey {
        channel: channel,
        cmd: cmd,
        args: args,
        input: input,
    };
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(result) = cache.get_mut(&key) {
            return Ok(result.clone());
        }

        let chan = match channel {
            ReleaseChannel::Stable => "stable",
            ReleaseChannel::Beta => "beta",
            ReleaseChannel::Nightly => "nightly",
        };

        let mut command = Command::new("playpen");
        command.arg(format!("root-{}", chan));
        command.arg("--mount-proc");
        command.arg("--user=rust");
        command.arg("--timeout=5");
        command.arg("--syscalls-file=whitelist");
        command.arg("--devices=/dev/urandom:r,/dev/null:rw");
        command.arg("--memory-limit=128");
        command.arg("--");
        command.arg(cmd);
        command.args(&key.args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());

        // Before `exec`ing playpen, redirect its stderr to stdout
        // There seems to be no simpler way of doing `2>&1` in Rust :((
        command.before_exec(|| {
            unsafe {
                assert_eq!(libc::dup2(libc::STDOUT_FILENO, libc::STDERR_FILENO), libc::STDERR_FILENO);
            }
            Ok(())
        });

        println!("running ({:?}): {} {:?}", channel, cmd, key.args);
        let mut child = try!(command.spawn());
        try!(child.stdin.as_mut().unwrap().write_all(key.input.as_bytes()));

        let out = try!(child.wait_with_output());
        println!("=> {}", out.status);
        cache.insert(key, (out.status.clone(), out.stdout.clone()));
        Ok((out.status.clone(), out.stdout.clone()))
    })
}

pub enum AsmFlavor {
    Att,
    Intel,
}

impl AsmFlavor {
    pub fn as_str(&self) -> &'static str {
        match *self {
            AsmFlavor::Att => "att",
            AsmFlavor::Intel => "intel",
        }
    }
}

impl FromStr for AsmFlavor {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "att" => Ok(AsmFlavor::Att),
            "intel" => Ok(AsmFlavor::Intel),
            _ => Err(StringError(format!("unknown asm dialect {}", s))),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

impl OptLevel {
    pub fn as_u8(&self) -> u8 {
        match *self {
            OptLevel::O0 => 0,
            OptLevel::O1 => 1,
            OptLevel::O2 => 2,
            OptLevel::O3 => 3,
        }
    }
}

impl FromStr for OptLevel {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(OptLevel::O0),
            "1" => Ok(OptLevel::O1),
            "2" => Ok(OptLevel::O2),
            "3" => Ok(OptLevel::O3),
            _ => Err(StringError(format!("unknown optimization level {}", s))),
        }
    }
}

pub enum CompileOutput {
    Asm,
    Llvm,
    Mir,
}

impl CompileOutput {
    pub fn as_opts(&self) -> &'static [&'static str] {
        // We use statics here since the borrow checker complains if we put these directly in the
        // match. Pretty ugly, but rvalue promotion might fix this.
        static ASM: &'static [&'static str] = &["--emit=asm"];
        static LLVM: &'static [&'static str] = &["--emit=llvm-ir"];
        static MIR: &'static [&'static str] = &["-Zunstable-options", "--unpretty=mir"];
        match *self {
            CompileOutput::Asm => ASM,
            CompileOutput::Llvm => LLVM,
            CompileOutput::Mir => MIR,
        }
    }
}

impl FromStr for CompileOutput {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asm" => Ok(CompileOutput::Asm),
            "llvm-ir" => Ok(CompileOutput::Llvm),
            "mir" => Ok(CompileOutput::Mir),
            _ => Err(StringError(format!("unknown output format {}", s))),
        }
    }
}

/// Highlights compiled rustc output according to the given output format
pub fn highlight(output_format: CompileOutput, output: &str) -> String {
    let lexer = match output_format {
        CompileOutput::Asm => "gas",
        CompileOutput::Llvm => "llvm",
        CompileOutput::Mir => return String::from(output),
    };

    let mut child = Command::new("pygmentize")
                            .arg("-l")
                            .arg(lexer)
                            .arg("-f")
                            .arg("html")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn().unwrap();
    write!(child.stdin.as_mut().unwrap(), "{}", output).unwrap();
    let output = child.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}
