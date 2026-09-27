/// Part of a backup system. This is server to
/// generate a hash for encryption, using the day as a salt.
/// A HTTP GET request can fetch the secret.
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::config::Config;
use crate::crypt::transform;
use crate::server::Server;

mod config;
mod crypt;
mod server;

fn main() {
    let config = Config::get().unwrap();

    let days_since_epoch = get_days_since_epoch();

    let cred = transform(config.key(), days_since_epoch.to_string());
    let content = format_content(days_since_epoch, cred);

    let mut server = Server::new(config.port(), content, config.api_keys.clone());

    let (tx, rx) = mpsc::channel();

    let server_thread = thread::spawn(move || {
        match server.serve(rx) {
            Ok(()) => {}
            Err(e) => eprintln!("An error occured: {e}"),
        };
    });

    thread::spawn(move || {
        loop {
            // Adding one second into the day to make sure we are really there.
            let time_until_midnight = Duration::from_secs(day_remaining_secs() + 1);
            thread::sleep(time_until_midnight);
            let content = format_content(
                get_days_since_epoch(),
                transform(config.key(), get_days_since_epoch().to_string()),
            );
            tx.send(content).unwrap()
        }
    });

    server_thread.join().unwrap()
}

fn format_content(date: u64, cred: String) -> String {
    format!("[credential]\ndate = {date}\ncredential = {cred}")
}

fn get_days_since_epoch() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / (24 * 60 * 60)
}

fn day_remaining_secs() -> u64 {
    (get_days_since_epoch() + 1) * (24 * 60 * 60)
        - SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::process::Command;

    #[test]
    fn rollover_works() {
        let content = format_content(1, "abc".to_string());

        let find_open_port_bind = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = find_open_port_bind.local_addr().unwrap().port();
        drop(find_open_port_bind);

        let mut server = Server::new(port, content.clone(), None);

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || server.serve(rx).unwrap());

        thread::sleep(Duration::from_millis(10));

        let result = Command::new("curl")
            .arg(format!("127.0.0.1:{}", port))
            .output()
            .unwrap();

        assert_eq!(content.as_bytes(), result.stdout.as_slice());

        thread::spawn(move || {
            loop {
                // Adding one second into the day to make sure we are really there.
                let time_until_midnight = Duration::from_millis(1);
                thread::sleep(time_until_midnight);
                let content = format_content(2, "def".to_string());
                tx.send(content).unwrap()
            }
        });

        thread::sleep(Duration::from_millis(10));

        let result = Command::new("curl")
            .arg(format!("127.0.0.1:{}", port))
            .output()
            .unwrap();

        assert_eq!(
            format_content(2, "def".to_string()).as_bytes(),
            result.stdout.as_slice()
        );
    }
}
