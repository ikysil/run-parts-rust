#[macro_use]
extern crate lazy_static;

use failure::{self, Error};
use io_mux::{Mux, TaggedData};
use is_executable::IsExecutable;
use regex::{Regex, RegexSet};

use std::fs;
use std::io::{self, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{self, Command};
use std::result::Result;
use structopt::StructOpt;

/// run scripts or programs in a directory
#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// print the names of the scripts which would be run, but don't actually run them.
    #[structopt(long)]
    test: bool,

    /// print the names of the all matching files (not limited to executables), but don't
    /// actually run them. This option cannot be used with --test.
    #[structopt(long)]
    list: bool,

    /// print the name of each script to stderr before running.
    #[structopt(short, long)]
    verbose: bool,

    /// similar to --verbose, but only prints the name of scripts which produce output.
    /// The script's name is printed to whichever of stdout or stderr the script produces
    /// output on. The script's name is not printed to stderr if --verbose also specified.
    #[structopt(long)]
    report: bool,

    /// reverse the scripts' execution order.
    #[structopt(long)]
    reverse: bool,

    /// exit as soon as a script returns with a non-zero exit code.
    #[structopt(long)]
    exit_on_error: bool,

    /// sets the umask to umask before running the scripts. umask should be specified in
    /// octal. By default the umask is set to 022.
    #[structopt(long, default_value = "022")]
    umask: String,

    /// filename must be in one or more of either the LANANA-assigned namespace, the LSB
    /// namespaces - either hierarchical or reserved - or the Debian cron script namespace.
    #[structopt(long)]
    lsbsysinit: bool,

    /// validate filenames against custom extended regular expression REGEX.
    #[structopt(long)]
    regex: Option<Regex>,

    /// pass argument to the scripts.  Use --arg once for each argument you want passed.
    #[structopt(short = "a", long)]
    arg: Vec<String>,

    #[structopt(name = "DIRECTORY", parse(from_os_str))]
    dir: PathBuf,
}

struct Status {
    pub exit_code: exitcode::ExitCode,
}

impl Status {
    pub fn new() -> Status {
        Status {
            exit_code: exitcode::OK,
        }
    }

    fn reset(&mut self) {
        self.exit_code = exitcode::OK;
    }
}

struct Report {
    report_string: String,
    report: bool,
    verbose: bool,
    used: bool
}

impl Report {
    fn new(opt: &Opt, fp: &PathBuf) -> Report {
        Report {
            report_string: String::from(fp.to_str().expect("cannot get file path")),
            report: opt.report,
            verbose: opt.verbose,
            used: false,
        }
    }

    fn get_report(self: &mut Self, condition: bool) -> Option<&String> {
        if self.used {
            return None;
        }
        self.used = true;
        if condition {
            Some(&self.report_string)
        } else {
            None
        }
    }

    fn out_report(self: &mut Self) -> Option<&String> {
        self.get_report(self.report)
    }

    fn err_report(self: &mut Self) -> Option<&String> {
        self.get_report(self.report && !self.verbose)
    }

}

fn usage_error(s: &str) {
    eprintln!("{}", s);
    eprintln!("");
    let app = Opt::clap();
    let mut out = io::stderr();
    app.write_help(&mut out).expect("failed to write to stderr");
    eprintln!("");
    process::exit(exitcode::USAGE)
}

fn find_files(opt: &Opt, dir: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut result: Vec<PathBuf> = [].to_vec();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        result.push(entry.path());
    }
    result.sort();
    if opt.reverse {
        result.reverse();
    }
    Ok(result)
}

const STD_SUFFIX_TO_IGNORE: [&str; 9] = [
    "~",
    ",",
    ".disabled",
    ".cfsaved",
    ".rpmsave",
    ".rpmorig",
    ".rpmnew",
    ".swp",
    ",v",
];

const LSBSYSINIT_SUFFIX_TO_IGNORE: [&str; 4] =
    [".dpkg-old", ".dpkg-dist", ".dpkg-new", ".dpkg-tmp"];

lazy_static! {
    static ref LSBSYSINIT_REGEX_TO_ACCEPT: RegexSet = RegexSet::new(&[
        r"^[a-z0-9]+$",                     // LANANA-assigned LSB hierarchical
        r"^_?([a-z0-9_.]+-)+[a-z0-9]+$",    // LANANA-assigned LSB reserved
        r"^[a-zA-Z0-9_-]+$"                 // Debian cron script namespaces
    ]).unwrap();
}

fn filter_filename(opt: &Opt, file_name: &str) -> bool {
    if STD_SUFFIX_TO_IGNORE
        .iter()
        .find(|&x| file_name.ends_with(x))
        .is_some()
    {
        return false;
    }
    if opt.lsbsysinit {
        if LSBSYSINIT_SUFFIX_TO_IGNORE
            .iter()
            .find(|&x| file_name.ends_with(x))
            .is_some()
        {
            return false;
        }
        if !LSBSYSINIT_REGEX_TO_ACCEPT.is_match(file_name) {
            return false;
        }
    }
    if let Some(regex) = &opt.regex {
        if !regex.is_match(file_name) {
            return false;
        }
    }
    true
}

fn filter_file(opt: &Opt, fp: &PathBuf) -> bool {
    if fp.as_path().is_dir() {
        return false;
    }
    if let Some(file_name) = fp.file_name().map(|x| x.to_str()) {
        return filter_filename(opt, &file_name.expect("cannot get file name"));
    } else {
        false
    }
}

fn act_on_file(opt: &Opt, fp: &PathBuf, status: &mut Status) {
    if opt.exit_on_error && status.exit_code != exitcode::OK {
        return;
    }
    status.reset();
    if opt.list {
        println!("{} {}", &fp.to_str().unwrap(), &opt.arg.join(" "));
        return;
    }
    if !fp.as_path().is_executable() {
        return;
    }
    if opt.test {
        println!("{} {}", &fp.to_str().unwrap(), &opt.arg.join(" "));
        return;
    }
    // TODO - implement random sleep
    if opt.verbose {
        eprintln!("{} {}", &fp.to_str().unwrap(), &opt.arg.join(" "));
    }
    // TODO - implement umask
    exec(opt, fp, status).unwrap();
    if opt.verbose {
        eprintln!(
            "{} {} exit status {}",
            &fp.to_str().unwrap(),
            &opt.arg.join(" "),
            status.exit_code
        );
    }
}

fn exec(opt: &Opt, fp: &PathBuf, status: &mut Status) -> Result<(), Error> {
    let mut mux = Mux::new()?;
    let mut report = Report::new(opt, fp);
    let mut child = Command::new(fp.to_str().unwrap())
        .args(&opt.arg)
        .stdout(mux.make_untagged_sender()?)
        .stderr(mux.make_tagged_sender("e")?)
        .spawn()?;
    let mut done_sender = mux.make_tagged_sender("d")?;
    std::thread::spawn(move || match child.wait() {
        Ok(status) => {
            let exit_code = if let Some(code) = status.code() {
                code as u8
            } else {
                status.signal().unwrap() as u8 + 128
            };
            let _ = done_sender.write_all(&[exit_code]);
        }
        Err(e) => {
            let _ = write!(done_sender, "Error: {:?}\n", e);
        }
    });

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    loop {
        let TaggedData { tag, data } = mux.read()?;
        match (tag.as_deref(), data) {
            (Some("d"), &[exit_code]) => {
                status.exit_code = exit_code as i32;
                break;
            },
            (Some("d"), error) => {
                std::io::stderr().write_all(error)?;
                status.exit_code = exitcode::SOFTWARE;
                break;
            }
            (None, _) => {
                write(&mut stdout, data, report.out_report())
            },
            (_, _) => {
                write(&mut stderr, data, report.err_report())
            },
        }
    }
    Ok(())
}

fn write(w: &mut dyn Write, data: &[u8], report: Option<&String>) {
    if let Some(report) = report {
        w.write_all(report.as_bytes()).unwrap();
        w.write_all("\n".as_bytes()).unwrap();
    }
    w.write_all(data).unwrap();
    w.write_all("\n".as_bytes()).unwrap();
}

fn run(opt: &Opt) -> Result<Status, Error> {
    let files = find_files(opt, &opt.dir)?;
    let files_to_process: Vec<&PathBuf> = files.iter().filter(|fp| filter_file(opt, fp)).collect();
    let mut status: Status = Status::new();
    for entry in files_to_process {
        act_on_file(opt, &entry, &mut status);
    }
    Ok(status)
}

#[cfg(debug_assertions)]
fn debug_options(opt: &Opt) {
    dbg!("{:#?}", &opt);
}

#[cfg(not(debug_assertions))]
fn debug_options(_opt: &Opt) {
}

fn main() {
    let opt = Opt::from_args();
    debug_options(&opt);
    if opt.list && opt.test {
        usage_error("--list and --test cannot be used together");
    }
    match run(&opt) {
        Ok(status) => process::exit(status.exit_code),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(exitcode::SOFTWARE);
        }
    }
}
