use std::path::Path;
use regex::RegexSet;

use crate::Opt;

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

lazy_static::lazy_static! {
    static ref LSBSYSINIT_REGEX_TO_ACCEPT: RegexSet = RegexSet::new(&[
        r"^[a-z0-9]+$",                     // LANANA-assigned LSB hierarchical
        r"^_?([a-z0-9_.]+-)+[a-z0-9]+$",    // LANANA-assigned LSB reserved
        r"^[a-zA-Z0-9_-]+$"                 // Debian cron script namespaces
    ]).unwrap();
}

fn filter_filename(opt: &Opt, file_name: &str) -> bool {
    if STD_SUFFIX_TO_IGNORE.iter().any(|&x| file_name.ends_with(x)) {
        return false;
    }
    if opt.lsbsysinit {
        if LSBSYSINIT_SUFFIX_TO_IGNORE
            .iter()
            .any(|&x| file_name.ends_with(x))
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

pub fn filter_file(opt: &Opt, fp: &Path) -> bool {
    if fp.is_dir() {
        return false;
    }
    if let Some(file_name) = fp.file_name().map(|x| x.to_str()) {
        filter_filename(opt, file_name.expect("cannot get file name"))
    } else {
        false
    }
}
