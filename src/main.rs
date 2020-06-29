use failure::{self, Error};
use is_executable::IsExecutable;

use std::fs;
use std::path::PathBuf;
use std::process;
use std::result::Result;
use structopt::StructOpt;

use run_parts::*;
use run_parts::exec::*;
use run_parts::filter::*;

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

fn run(opt: &Opt) -> Result<Status, Error> {
    let files = find_files(opt, &opt.dir)?;
    let files_to_process: Vec<&PathBuf> = files.iter().filter(|fp| filter_file(opt, fp)).collect();
    let mut status: Status = Status::new();
    for entry in files_to_process {
        act_on_file(opt, &entry, &mut status);
    }
    Ok(status)
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
