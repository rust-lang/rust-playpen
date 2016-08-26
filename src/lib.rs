#[macro_use]
extern crate log;
extern crate libc;
extern crate lru_cache;
extern crate wait_timeout;

use lru_cache::LruCache;

use std::error::Error;
use std::fmt;
use std::io::Write;
use std::io;
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;

use docker::Container;

mod docker;

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

pub struct Cache {
    cache: Mutex<LruCache<CacheKey, (ExitStatus, Vec<u8>)>>,
}

#[derive(PartialEq, Eq, Hash)]
struct CacheKey {
    channel: ReleaseChannel,
    cmd: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    input: String,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            cache: Mutex::new(LruCache::new(256)),
        }
    }

    /// Helper method for safely invoking a command inside a playpen
    pub fn exec(&self,
                channel: ReleaseChannel,
                cmd: &str,
                args: Vec<String>,
                env: Vec<(String, String)>,
                input: String)
                -> io::Result<(ExitStatus, Vec<u8>)> {
        // Build key to look up
        let key = CacheKey {
            channel: channel,
            cmd: cmd.to_string(),
            args: args,
            env: env,
            input: input,
        };
        let mut cache = self.cache.lock().unwrap();
        if let Some(prev) = cache.get_mut(&key) {
            return Ok(prev.clone())
        }
        drop(cache);

        let chan = match channel {
            ReleaseChannel::Stable => "stable",
            ReleaseChannel::Beta => "beta",
            ReleaseChannel::Nightly => "nightly",
        };
        let container = format!("rust-playpen-{}", chan);

        let container = try!(Container::new(cmd, &key.args, &key.env, &container));

        let tuple = try!(container.run(key.input.as_bytes(), Duration::new(5, 0)));
        let (status, mut output, timeout) = tuple;
        if timeout {
            output.extend_from_slice(b"\ntimeout triggered!");
        }
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, (status.clone(), output.clone()));
        Ok((status, output))
    }
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
pub enum Backtrace {
    Never,
    Always,
    Auto,
}

impl Backtrace {
    pub fn is_requested(&self, debug: bool) -> bool {
        match *self {
            Backtrace::Never => false,
            Backtrace::Always => true,
            Backtrace::Auto => debug,
        }
    }
}

impl FromStr for Backtrace {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Backtrace::Never),
            "1" => Ok(Backtrace::Always),
            "2" => Ok(Backtrace::Auto),
            _ => Err(StringError(format!("unknown backtrace setting {}", s))),
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
        CompileOutput::Mir => "text",
    };

    let mut child = Command::new("pygmentize")
                            .arg("-l")
                            .arg(lexer)
                            .arg("-f")
                            .arg("html")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn().unwrap();
    child.stdin.take().unwrap().write_all(output.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use super::*;

    #[test]
    fn eval() {
        drop(env_logger::init());

        let cache = Cache::new();
        let input = r#"fn main() { println!("Hello") }"#;
        let (status, out) = cache.exec(ReleaseChannel::Stable,
                                       "/usr/local/bin/evaluate.sh",
                                       Vec::new(),
                                       Vec::new(),
                                       input.to_string()).unwrap();
        assert!(status.success());
        assert_eq!(out, &[0xff, b'H', b'e', b'l', b'l', b'o', b'\n']);
    }

    #[test]
    fn timeout() {
        drop(env_logger::init());

        let cache = Cache::new();
        let input = r#"
            fn main() {
                std::thread::sleep_ms(10_000);
            }
        "#;
        let (status, out) = cache.exec(ReleaseChannel::Stable,
                                       "/usr/local/bin/evaluate.sh",
                                       Vec::new(),
                                       Vec::new(),
                                       input.to_string()).unwrap();
        assert!(!status.success());
        assert!(String::from_utf8_lossy(&out).contains("timeout triggered"));
    }

    #[test]
    fn compile() {
        drop(env_logger::init());

        let cache = Cache::new();
        let input = r#"fn main() { println!("Hello") }"#;
        let (status, out) = cache.exec(ReleaseChannel::Stable,
                                       "/usr/local/bin/compile.sh",
                                       vec![String::from("--emit=llvm-ir")],
                                       vec![],
                                       input.to_string()).unwrap();

        assert!(status.success());
        let mut split = out.splitn(2, |b| *b == b'\xff');
        let empty: &[u8] = &[];
        assert_eq!(split.next().unwrap(), empty);
        assert!(String::from_utf8(split.next().unwrap().to_vec()).unwrap()
            .contains("target triple"));
    }

    #[test]
    fn fmt() {
        drop(env_logger::init());

        let cache = Cache::new();
        let input = r#"fn main() { println!("Hello") }"#;
        let (status, out) = cache.exec(ReleaseChannel::Stable,
                                       "rustfmt",
                                       Vec::new(),
                                       Vec::new(),
                                       input.to_string()).unwrap();
        assert!(status.success());
        assert!(String::from_utf8(out).unwrap().contains(r#""Hello""#))
    }

    #[test]
    fn pygmentize() {
        drop(env_logger::init());

        assert!(highlight(CompileOutput::Llvm, "target triple").contains("<span"));
    }
}
