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
        fs::create_dir_all(&tool_path).expect("Falha ao criar o diretório de backup.");
    }

    if args.stats {
        println!("{}", "=== Status do Purrgres ===".bold().underline());

        match utils::process::check_process_status() {
            Some(pid) => {
                let elapsed_time = utils::process::get_process_uptime(pid);
                println!("Backup em execução: {}", format!("PID: {}", pid).green());
                println!("Tempo de execução: {}", elapsed_time.yellow());
            }
            None => {
                println!("{}", "Backup não está em execução.".red());
            }
        }

        println!("{}", "=".repeat(25).bold());

        return;
    }

    if args.stop {
        println!("=== Parando o Backup ===");
        match utils::process::stop_process() {
            Ok(_) => {
                println!("Processo de backup parado.");
                utils::process::clear_pid();
            }
            Err(e) => eprintln!("Erro ao parar o processo: {}", e),
        }
        println!("=========================");
        return;
    }

    if args.list_purrs {
        utils::process::list_backups(&tool_path);
        return;
    }

    if let Some(backup_file) = args.rpurry.as_ref() {
        println!("=== Restaurando backup de: {} === ", backup_file);

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
                &args
                    .container
                    .clone()
                    .expect("Necessário prover nome do container"),
                "pg_dump",
                "-U",
                &args
                    .user
                    .clone()
                    .expect("Necessario prover o nome do usuário"),
                &args.database.clone().expect("Necessário prover o banco"),
            ])
            .output()
            .expect("Falha ao executar o comando pg_dump");

        let pid = std::process::id();
        utils::process::save_pid(pid);

        if output.status.success() {
            fs::write(&file_name, output.stdout).expect("Falha ao salvar o arquivo de backup");
            println!("=== Backup Concluído ===");
            println!("Backup salvo em {}", file_name);
        } else {
            eprintln!(
                "Erro ao realizar o backup: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
