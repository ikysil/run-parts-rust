use regex::Regex;
use structopt::StructOpt;

use std::io;
use std::path::PathBuf;
use std::process;

pub mod exec;
pub mod filter;

/// run scripts or programs in a directory
#[derive(StructOpt, Debug)]
#[structopt()]
pub struct Opt {
    /// print the names of the scripts which would be run, but don't actually run them.
    #[structopt(long)]
    pub test: bool,

    /// print the names of the all matching files (not limited to executables), but don't
    /// actually run them. This option cannot be used with --test.
    #[structopt(long)]
    pub list: bool,

    /// print the name of each script to stderr before running.
    #[structopt(short, long)]
    pub verbose: bool,

    /// similar to --verbose, but only prints the name of scripts which produce output.
    /// The script's name is printed to whichever of stdout or stderr the script produces
    /// output on. The script's name is not printed to stderr if --verbose also specified.
    #[structopt(long)]
    pub report: bool,

    /// reverse the scripts' execution order.
    #[structopt(long)]
    pub reverse: bool,

    /// exit as soon as a script returns with a non-zero exit code.
    #[structopt(long)]
    pub exit_on_error: bool,

    /// sets the umask to umask before running the scripts. umask should be specified in
    /// octal. By default the umask is set to 022.
    #[structopt(long, default_value = "022")]
    pub umask: String,

    /// filename must be in one or more of either the LANANA-assigned namespace, the LSB
    /// namespaces - either hierarchical or reserved - or the Debian cron script namespace.
    #[structopt(long)]
    pub lsbsysinit: bool,

    /// validate filenames against custom extended regular expression REGEX.
    #[structopt(long)]
    pub regex: Option<Regex>,

    /// pass argument to the scripts.  Use --arg once for each argument you want passed.
    #[structopt(short = "a", long)]
    pub arg: Vec<String>,

    #[structopt(name = "DIRECTORY", parse(from_os_str))]
    pub dir: PathBuf,
}

impl Opt {
    pub fn usage_error(self: &Self, s: &str) {
        eprintln!("{}", s);
        eprintln!("");
        let app = Self::clap();
        let mut out = io::stderr();
        app.write_help(&mut out).expect("failed to write to stderr");
        eprintln!("");
        process::exit(exitcode::USAGE)
    }

    #[cfg(debug_assertions)]
    pub fn debug_options(self: &Self) {
        dbg!("{:#?}", &self);
    }

    #[cfg(not(debug_assertions))]
    pub fn debug_options(self: &Self) {}
}

#[derive(Default)]
pub struct Status {
    pub exit_code: exitcode::ExitCode,
}

impl Status {
    pub fn reset(&mut self) {
        self.exit_code = exitcode::OK;
    }
}

pub struct Report {
    report_string: String,
    report: bool,
    verbose: bool,
    used: bool,
}

impl Report {
    pub fn new(opt: &Opt, fp: &PathBuf) -> Report {
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

    pub fn out_report(self: &mut Self) -> Option<&String> {
        self.get_report(self.report)
    }

    pub fn err_report(self: &mut Self) -> Option<&String> {
        self.get_report(self.report && !self.verbose)
    }
}
