//! One JSON request/response per line on private stdio; usable without LazyBuddy.
#![cfg_attr(windows, windows_subsystem = "windows")]
use lazybuddy_news::{feeds, protocol};
use std::io::{self, BufRead, Read, Write};

fn main() -> io::Result<()> {
    let client = feeds::client();
    let mut service =
        lazybuddy_news::state::Service::new(lazybuddy_news::state::Service::default_path());
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut line = Vec::new();
        let mut limited = (&mut input).take((protocol::MAX_REQUEST_BYTES + 2) as u64);
        if limited.read_until(b'\n', &mut line)? == 0 {
            return Ok(());
        }
        if line.last() == Some(&b'\n') {
            line.pop();
        }
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        let oversized = line.len() > protocol::MAX_REQUEST_BYTES;
        let reply = service.handle(&client, &line);
        serde_json::to_writer(&mut output, &reply)?;
        writeln!(output)?;
        output.flush()?;
        if oversized {
            return Ok(());
        }
    }
}
