use chrono::Utc;
use cron::Schedule;
use std::{str::FromStr, thread};
use tracing_appender::{non_blocking::WorkerGuard, rolling::daily};
use tracing_subscriber::{filter, fmt, prelude::*, util::SubscriberInitExt};

pub fn initialize_logger(file_name: &str) -> WorkerGuard {

    let file_appender = daily("./log", file_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer().with_writer(non_blocking).json().pretty();

    let stdout_log = tracing_subscriber::fmt::layer().compact();
    tracing_subscriber::registry().with(filter::LevelFilter::INFO).with(stdout_log).with(file_layer).init();
       
    let file_name_thread = String::from(file_name);
    create_symlink_file(file_name_thread.clone());

    thread::spawn(move ||{
        let midnight = "0 0 0 * * *";
        let schedule = Schedule::from_str(midnight).unwrap();
        loop {
            let now = Utc::now();
            if let Some(next) = schedule.upcoming(Utc).take(1).next() {
                let until_next = next - now;
                thread::sleep(until_next.to_std().unwrap());
                create_symlink_file(file_name_thread.clone());
            }
        }
    });
    _guard
}


fn create_symlink_file(file_name_thread:String)  {
    let today = Utc::now().format("%Y-%m-%d").to_string();
        let log_file_name = format!("{}.{}",file_name_thread.clone(), today);
        let latest_log_file_name = format!("latest_{}",file_name_thread);
        let symlink_path = format!("./log/{}", latest_log_file_name);
        let symlink_path = std::path::Path::new(&symlink_path);
        if symlink_path.exists() {
            std::fs::remove_file(symlink_path).expect("Failed to remove existing symlink");
        }
        std::os::unix::fs::symlink(log_file_name, symlink_path).expect("Failed to create symlink");
}
