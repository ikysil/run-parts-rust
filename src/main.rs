use std::path::PathBuf;
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

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
}
