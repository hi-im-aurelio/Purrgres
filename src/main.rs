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
    #[arg(short, long, help = "Usuário do PostgreSQL", required_unless_present_any = ["stats", "stop"])]
    user: Option<String>,

    #[arg(short, long, help = "Nome do banco de dados", required_unless_present_any = ["stats", "stop"])]
    database: Option<String>,

    #[arg(short, long, help = "Nome do container", required_unless_present_any = ["stats", "stop"])]
    container: Option<String>,

    #[arg(long, help = "Verifica o status do backup em execução.")]
    stats: bool,

    #[arg(long, help = "Para o processo de backup")]
    stop: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let tool_path = utils::path::get_bkp_path();

    if !tool_path.exists() {
        fs::create_dir_all(&tool_path).expect("Falha ao criar o diretório de backup.");
    }

    if args.stats {
        match utils::process::check_process_status() {
            Some(pid) => println!("Backup está em execução (PID: {})", pid),
            None => println!("Backup não está em execução."),
        }
        return;
    }

    if args.stop {
        match utils::process::stop_process() {
            Ok(_) => println!("Processo de backup parado."),
            Err(e) => eprintln!("Erro ao parar o processo: {}", e),
        }
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
            println!("Backup salvo em {}", file_name);
        } else {
            eprintln!(
                "Erro ao realizar o backup: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // utils::process::clear_pid();
    }
}
