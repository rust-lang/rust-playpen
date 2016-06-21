extern crate rust_playpen;

#[macro_use] extern crate iron;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate hyper;
extern crate staticfile;
extern crate router;
extern crate unicase;
extern crate rustc_serialize;

use rust_playpen::*;

use iron::prelude::*;
use iron::status;
use iron::headers;
use iron::middleware::AfterMiddleware;
use iron::modifiers::Header;
use iron::method::Method;
use hyper::header;
use staticfile::Static;
use router::Router;
use unicase::UniCase;
use rustc_serialize::json;

use std::fmt;
use std::path::Path;
use std::io::Read;
use std::process::Command;

#[derive(Clone, Debug)]
struct XXssProtection(bool);

impl header::Header for XXssProtection {
    fn header_name() -> &'static str {
        "X-XSS-Protection"
    }

    fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<Self> {
        if raw.len() == 1 {
            let line = &raw[0];
            if line.len() == 1 {
                let byte = line[0];
                match byte {
                    b'1' => return Ok(XXssProtection(true)),
                    b'0' => return Ok(XXssProtection(false)),
                    _ => ()
                }
            }
        }
        Err(hyper::Error::Header)
    }
}

impl header::HeaderFormat for XXssProtection {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 {
            f.write_str("1")
        } else {
            f.write_str("0")
        }
    }
}

fn index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((Path::new("static/web.html"),
                       Header(XXssProtection(false)))))
}

/// The JSON-encoded request sent to `evaluate.json`.
#[derive(RustcDecodable)]
struct EvaluateReq {
    color: Option<bool>,
    test: Option<bool>,
    version: Option<String>,
    optimize: Option<String>,
    separate_output: Option<bool>,
    code: String,
}

fn evaluate(req: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    itry!(req.body.read_to_string(&mut body));

    let data: EvaluateReq = itry!(json::decode(&body));
    let color = data.color.unwrap_or(false);
    let test = data.test.unwrap_or(false);
    let version = itry!(data.version.map(|v| v.parse()).unwrap_or(Ok(ReleaseChannel::Stable)));
    let opt = itry!(data.optimize.map(|opt| opt.parse()).unwrap_or(Ok(OptLevel::O2)));
    let separate_output = data.separate_output.unwrap_or(false);

    let mut args = vec![String::from("-C"), format!("opt-level={}", opt.as_u8())];
    if opt == OptLevel::O0 {
        args.push(String::from("-g"));
    }
    if color {
        args.push(String::from("--color=always"));
    }
    if test {
        args.push(String::from("--test"));
    }

    let (_status, output) = itry!(
        rust_playpen::exec(version, "/usr/local/bin/evaluate.sh", args, data.code));

    let mut obj = json::Object::new();
    if separate_output {
        // {"rustc": "...", "program": "..."}
        let mut split = output.splitn(2, |b| *b == b'\xff');
        let rustc = String::from_utf8(split.next().unwrap().into()).unwrap();

        obj.insert(String::from("rustc"), json::Json::String(rustc));

        if let Some(program) = split.next() {
            // Compilation succeeded
            let output = String::from_utf8_lossy(program).into_owned();
            obj.insert(String::from("program"), json::Json::String(output));
        }
    } else {
        // {"result": "...""}
        let result = output.splitn(2, |b| *b == b'\xff')
                           .map(|sub| String::from_utf8_lossy(sub).into_owned())
                           .collect::<String>();

        obj.insert(String::from("result"), json::Json::String(result));
    }

    Ok(Response::with((status::Ok, format!("{}", json::Json::Object(obj)))))
}

#[derive(RustcDecodable)]
struct CompileReq {
    syntax: Option<String>,
    color: Option<bool>,
    version: Option<String>,
    optimize: Option<String>,
    emit: Option<String>,
    code: String,
}

fn compile(req: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    itry!(req.body.read_to_string(&mut body));

    let data: CompileReq = itry!(json::decode(&body));
    let syntax = data.syntax.map(|s| s.parse().unwrap()).unwrap_or(AsmFlavor::Att);
    let color = data.color.unwrap_or(false);
    let version = itry!(data.version.map(|v| v.parse()).unwrap_or(Ok(ReleaseChannel::Stable)));
    let opt = itry!(data.optimize.map(|opt| opt.parse()).unwrap_or(Ok(OptLevel::O2)));
    let emit = itry!(data.emit.map(|emit| emit.parse()).unwrap_or(Ok(CompileOutput::Asm)));

    let mut args = vec![
        String::from("-C"),
        format!("opt-level={}", opt.as_u8()),
        String::from("-C"),
        format!("llvm-args=-x86-asm-syntax={}", syntax.as_str()),
    ];
    for opt in emit.as_opts() {
        args.push(String::from(*opt));
    }
    if opt == OptLevel::O0 {
        args.push(String::from("-g"));
    }
    if color {
        args.push(String::from("--color=always"));
    }

    let (_status, output) = itry!(
        rust_playpen::exec(version, "/usr/local/bin/compile.sh", args, data.code));
    let mut split = output.splitn(2, |b| *b == b'\xff');
    let rustc = String::from_utf8(split.next().unwrap().into()).unwrap();

    let mut obj = json::Object::new();
    match split.next() {
        Some(program_out) => {
            // Compilation succeeded
            let output = highlight(emit,
                                   &String::from_utf8_lossy(program_out).into_owned());
            obj.insert(String::from("result"), json::Json::String(output));
        }
        None => {
            obj.insert(String::from("error"), json::Json::String(rustc));
        }
    }

    Ok(Response::with((status::Ok, format!("{}", json::Json::Object(obj)))))
}

#[derive(RustcDecodable)]
struct FormatReq {
    version: Option<String>,
    code: String,
}

fn format(req: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    req.body.read_to_string(&mut body).unwrap();

    let data: FormatReq = itry!(json::decode(&body));
    let version = itry!(data.version.map(|v| v.parse()).unwrap_or(Ok(ReleaseChannel::Stable)));

    let (status, output) = itry!(
        rust_playpen::exec(version, "/usr/bin/rustfmt", Vec::new(), data.code));
    let output = String::from_utf8(output).unwrap();
    let mut response_obj = json::Object::new();
    if status.success() {
        response_obj.insert(String::from("result"), json::Json::String(output));
    } else {
        response_obj.insert(String::from("error"), json::Json::String(output));
    }

    Ok(Response::with((status::Ok, format!("{}", json::Json::Object(response_obj)))))
}

// This is neat!
struct EnablePostCors;
impl AfterMiddleware for EnablePostCors {
    fn after(&self, _: &mut Request, res: Response) -> IronResult<Response> {
        Ok(res.set(Header(headers::AccessControlAllowOrigin::Any))
              .set(Header(headers::AccessControlAllowMethods(
                  vec![Method::Post,
                       Method::Options])))
              .set(Header(headers::AccessControlAllowHeaders(
                  vec![UniCase(String::from("Origin")),
                       UniCase(String::from("Accept")),
                       UniCase(String::from("Content-Type"))]))))
    }
}

fn main() {
    env_logger::init().unwrap();

    // Make sure pygmentize is installed before starting the server
    Command::new("pygmentize").spawn().unwrap().kill().unwrap();

    let mut router = Router::new();
    router.get("/", index);
    router.get("/:path", Static::new("static"));
    router.post("/evaluate.json", evaluate);
    router.post("/compile.json", compile);
    router.post("/format.json", format);

    // Use our router as the middleware, and pass the generated response through `EnablePostCors`
    let mut chain = Chain::new(router);
    chain.link_after(EnablePostCors);

    let addr = ("0.0.0.0", 80);
    info!("listening on {:?}", addr);
    Iron::new(chain).http(addr).unwrap();
}
