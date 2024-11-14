use chrono::Local;
use clap::Parser;
use colored::*;
use std::fs;
use std::process::Command;
use std::time::Duration;
use tokio::time;

mod utils;

#[tokio::main]
async fn main() {
    let args = utils::args_struct::Args::parse();

    let tool_path = utils::path::get_bkp_path();

    if !tool_path.exists() {
        fs::create_dir_all(&tool_path).expect("Failed to create backup directory");
    }

    if args.stats {
        println!("{}", "=== Status purrgres ===".bold().underline());

        match utils::process::check_process_status() {
            Some(pid) => {
                let elapsed_time = utils::process::get_process_uptime(pid);
                println!("Backup running: {}", format!("PID: {}", pid).green());
                println!("Execution time: {}", elapsed_time.yellow());
            }
            None => {
                println!("{}", "Backup is not running".red());
            }
        }

        println!("{}", "=".repeat(25).bold());

        return;
    }

    if args.stop {
        println!("=== Stopping the backup ===");
        match utils::process::stop_process() {
            Ok(_) => {
                println!("Backup process stopped");
                utils::process::clear_pid();
            }
            Err(e) => eprintln!("Error stopping the process: {}", e),
        }
        println!("=========================");
        return;
    }

    if args.list_purrs {
        utils::process::list_backups(&tool_path);
        return;
    }

    if let Some(backup_file) = args.rpurry.as_ref() {
        println!("=== Restoring backup from: {} === ", backup_file);

        utils::process::apply_backup(backup_file, &args);

        println!("=========================");
        return;
    }

    let mut interval = time::interval(Duration::from_secs(86400));

    loop {
        interval.tick().await;

        let now = Local::now();
        let file_name = format!(
            "{}/{}_backup.sql",
            tool_path.to_str().expect("Failed to get backup folder."),
            now.format("%d_%m_%Y_%H_%M"),
        );

        let output = Command::new("docker")
            .args(&[
                "exec",
                &args.container.clone().expect("Container name required"),
                "pg_dump",
                "-U",
                &args.user.clone().expect("Database user required"),
                &args.database.clone().expect("Database name required"),
            ])
            .output()
            .expect("Failed to execute the pg_dump command");

        let pid = std::process::id();
        utils::process::save_pid(pid);

        if output.status.success() {
            fs::write(&file_name, output.stdout).expect("Failed save the backup file");
            println!("=== Backup complete ===");
            println!("Backup saved in: {}", file_name);
        } else {
            eprintln!(
                "Error make backup: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
