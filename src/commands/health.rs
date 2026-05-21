use crate::output::HealthResult;
use anyhow::Result;
use std::io::{Read, Write};
use std::net::TcpStream;

pub async fn run(cmd: &str, host: &str) -> Result<HealthResult> {
    let addr = if host.contains(':') {
        host.to_string()
    } else {
        format!("{}:2181", host)
    };
    let response = send_4lw(&addr, cmd)?;
    Ok(HealthResult {
        command: cmd.to_string(),
        host: host.to_string(),
        output: response,
    })
}

pub fn format_human(r: &HealthResult) -> String {
    r.output.clone()
}

fn send_4lw(addr: &str, cmd: &str) -> Result<String> {
    let mut stream = TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    stream.write_all(cmd.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}
