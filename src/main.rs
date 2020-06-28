use failure::{self, Error, Fail};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;
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
    regex: Option<String>,

    /// pass argument to the scripts.  Use --arg once for each argument you want passed.
    #[structopt(short = "a", long)]
    arg: Vec<String>,

    #[structopt(name = "DIRECTORY", parse(from_os_str))]
    dir: PathBuf,
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

fn find_files(opt: &Opt, dir: &PathBuf) -> Result<Vec<PathBuf>, Error>{
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

fn run(opt: &Opt) -> Result<(), Error> {
    let files = find_files(opt, &opt.dir)?;
    for entry in files {
        println!("file: {:?}", &entry);
    }
    Ok(())
}

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
    if opt.list && opt.test {
        usage_error("--list and --test cannot be used together");
    }
    match run(&opt) {
        Ok(_) => process::exit(exitcode::OK),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(exitcode::SOFTWARE);
        }
    }
}
