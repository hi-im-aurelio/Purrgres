use chrono::Local;
use clap::{command, Parser};
use std::fs;
use std::process::Command;
use std::time::Duration;
use tokio::time;

mod utils;

#[derive(Parser, Debug)]
#[command(name = "purrgres")]
#[command(about = "Uma ferramenta de backup automática para PostgreSQL em contêineres Docker.")]
struct Args {
    #[arg(short, long, help = "Usuário do PostgreSQL")]
    user: String,

    #[arg(short, long, help = "Nome do banco de dados")]
    database: String,

    #[arg(short, long, help = "Nome do container")]
    container: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let tool_path = utils::path::get_bkp_path();

    if !tool_path.exists() {
        fs::create_dir_all(&tool_path).expect("Falha ao criar o diretório de backup.");
    }

    let mut interval = time::interval(Duration::from_secs(86400));

    loop {
        interval.tick().await;

        let now = Local::now();
        let file_name = format!(
            "{}/{}_backup.sql",
            tool_path.to_str().expect("Failed to get backup folder.",),
            now.format("%d_%m_%Y_%H_%M"),
        );

        let status = Command::new("docker")
            .args(&[
                "exec",
                &args.container,
                "pg_dump",
                "-U",
                &args.user,
                &args.database,
            ])
            .output()
            .expect("Falha ao executar o comando pg_dump");

        if status.status.success() {
            fs::write(&file_name, status.stdout).expect("Falha ao salvar o arquivo de backup");
            println!("Backup salvo em {}", file_name);
        } else {
            eprintln!("Erro ao realizar o backup: {:?}", status.stderr);
        }
    }
}
