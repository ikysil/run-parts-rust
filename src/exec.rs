use std::path::Path;
use failure::{self, Error};
use io_mux::{Mux, TaggedData};

use std::io::{self, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::Command;

use crate::{Opt, Report, Status};

pub fn exec(opt: &Opt, fp: &Path, status: &mut Status) -> Result<(), Error> {
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
            let _ = writeln!(done_sender, "Error: {:?}", e);
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
            }
            (Some("d"), error) => {
                std::io::stderr().write_all(error)?;
                status.exit_code = exitcode::SOFTWARE;
                break;
            }
            (None, _) => write(&mut stdout, data, report.out_report()),
            (_, _) => write(&mut stderr, data, report.err_report()),
        }
    }
    Ok(())
}

fn write(w: &mut dyn Write, data: &[u8], report: Option<&String>) {
    if let Some(report) = report {
        w.write_all(report.as_bytes()).unwrap();
        w.write_all(b":\n").unwrap();
    }
    w.write_all(data).unwrap();
}
