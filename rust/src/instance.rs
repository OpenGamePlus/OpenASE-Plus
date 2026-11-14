//! One running OpenASE: a second instance started with an `openase://` link
//! hands the link to the first one and exits.
//!
//! The first instance listens on a random `127.0.0.1` port and writes it to
//! a file in the config folder. A link is sent as one line, `OPENASE1 <url>`,
//! answered `OK`. Only valid links ([`crate::links::parse`]) are accepted on
//! either side, and the GUI still asks before joining, so a local process
//! that writes to the port can do no more than starting OpenASE with a link.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::links;

const HELLO: &str = "OPENASE1 ";
const TIMEOUT: Duration = Duration::from_secs(2);

/// File holding the listening port.
pub fn port_file() -> PathBuf {
    crate::settings::config_dir().join("instance.port")
}

/// Hand `url` to the running instance; false when none answers.
pub fn forward(port_file: &Path, url: &str) -> bool {
    if links::parse(url).is_none() {
        return false;
    }
    let Some(port) = std::fs::read_to_string(port_file)
        .ok()
        .and_then(|text| text.trim().parse::<u16>().ok())
    else {
        return false;
    };
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, TIMEOUT) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(TIMEOUT));
    if writeln!(stream, "{HELLO}{url}").is_err() {
        return false;
    }
    let mut answer = String::new();
    BufReader::new(stream).read_line(&mut answer).is_ok() && answer.trim() == "OK"
}

/// Become the running instance: accept links in a background thread and pass
/// each valid one to `on_link`.
pub fn listen(port_file: &Path, on_link: impl Fn(String) + Send + 'static) -> std::io::Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    if let Some(dir) = port_file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(port_file, listener.local_addr()?.port().to_string())?;
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = stream.set_read_timeout(Some(TIMEOUT));
            let mut line = String::new();
            let mut reader = BufReader::new((&stream).take(512));
            let valid = reader.read_line(&mut line).is_ok()
                && line
                    .trim_end()
                    .strip_prefix(HELLO)
                    .is_some_and(|url| links::parse(url).is_some());
            let mut stream = &stream;
            if valid {
                on_link(line.trim_end()[HELLO.len()..].to_string());
                let _ = stream.write_all(b"OK\n");
            } else {
                let _ = stream.write_all(b"NO\n");
            }
        }
    });
    Ok(())
}
