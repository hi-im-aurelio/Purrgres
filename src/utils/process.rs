use chrono::NaiveDateTime;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use super::args_struct::Args;
use crate::utils::path::get_bkp_path;

fn get_pid_file_path() -> PathBuf {
    get_bkp_path().join("purrgres_pid")
}

pub fn stop_process() -> Result<(), String> {
    if let Some(pid) = check_process_status() {
        let output = Command::new("kill")
            .arg(pid.to_string())
            .output()
            .map_err(|e| format!("Falha ao parar o processo: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err("Falha ao matar o processo.".to_string())
        }
    } else {
        Err("Nenhum processo de backup encontrado.".to_string())
    }
}

pub fn check_process_status() -> Option<u32> {
    if let Ok(pid_str) = fs::read_to_string(get_pid_file_path()) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            return Some(pid);
        }
    }
    None
}

pub fn clear_pid() {
    fs::remove_file(get_pid_file_path()).expect("Falha ao remover arquivo PID");
}

pub fn save_pid(pid: u32) {
    let pid_file_path = get_bkp_path().join("purrgres_pid");
    let mut file = File::create(&pid_file_path)
        .map_err(|e| format!("Falha ao criar o arquivo PID: {}", e))
        .expect("Erro ao criar o arquivo PID");

    write!(file, "{}", pid).expect("Falha ao escrever PID no arquivo");

    println!("Salvando PID: {}", pid);
}

pub fn get_process_uptime(pid: u32) -> String {
    let start_time = Command::new("ps")
        .args(&["-p", &pid.to_string(), "-o", "lstart="])
        .output()
        .expect("Falha ao obter o tempo de início do processo");

    if start_time.status.success() {
        let start_time_str = String::from_utf8_lossy(&start_time.stdout);

        let months = [
            ("jan", "Jan"),
            ("fev", "Feb"),
            ("mar", "Mar"),
            ("abr", "Apr"),
            ("mai", "May"),
            ("jun", "Jun"),
            ("jul", "Jul"),
            ("ago", "Aug"),
            ("set", "Sep"),
            ("out", "Oct"),
            ("nov", "Nov"),
            ("dez", "Dec"),
        ];

        let days_of_week = [
            ("dom", "Sun"),
            ("seg", "Mon"),
            ("ter", "Tue"),
            ("qua", "Wed"),
            ("qui", "Thu"),
            ("sex", "Fri"),
            ("sab", "Sat"),
        ];

        let mut start_time_str = start_time_str.to_string();
        for (pt, en) in &days_of_week {
            start_time_str = start_time_str.replace(*pt, *en);
        }
        for (pt, en) in &months {
            start_time_str = start_time_str.replace(*pt, *en);
        }

        let start_time =
            chrono::NaiveDateTime::parse_from_str(&start_time_str.trim(), "%a %b %d %H:%M:%S %Y");

        match start_time {
            Ok(start_time) => {
                let elapsed_time = chrono::Local::now().naive_local() - start_time;

                let total_seconds = elapsed_time.num_seconds();
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                format!("{}h {}m {}s", hours, minutes, seconds)
            }
            Err(_) => "Não foi possível calcular o tempo.".to_string(),
        }
    } else {
        "Não foi possível calcular o tempo.".to_string()
    }
}

pub fn apply_backup(backup_file: &str, args: &Args) {
    let copy_output = Command::new("docker")
        .args(&[
            "cp",
            get_bkp_path()
                .join(backup_file)
                .to_str()
                .expect("Falha ao obter o caminho do arquivo de backup"),
            &format!(
                "{}:/tmp/{}",
                args.container
                    .clone()
                    .expect("Necessário prover nome do container"),
                backup_file
            ),
        ])
        .output()
        .expect("Falha ao copiar o arquivo de backup para o contêiner");

    if !copy_output.status.success() {
        eprintln!(
            "Erro ao copiar o arquivo para o contêiner: {:?}",
            String::from_utf8_lossy(&copy_output.stderr)
        );
        return;
    }

    let output = Command::new("docker")
        .args(&[
            "exec",
            &args
                .container
                .clone()
                .expect("Necessário prover nome do container"),
            "psql",
            "-U",
            &args
                .user
                .clone()
                .expect("Necessário prover nome do usuário"),
            "-d",
            &args
                .database
                .clone()
                .expect("Necessário prover nome do banco"),
            "-f",
            &format!("/tmp/{}", backup_file), // isto deve usar o caminho do proprio container.
        ])
        .output()
        .expect("Falha ao executar o comando psql");

    if output.status.success() {
        println!("Backup restaurado com sucesso!");

        let log_file = get_bkp_path().join(".purrs");
        let mut log = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file)
            .expect("Falha ao abrir o arquivo de log");

        let now = chrono::Local::now();
        writeln!(
            log,
            "Backup restaurado de: {} - Data: {}",
            backup_file,
            now.format("%d/%m/%Y %H:%M")
        )
        .expect("Falha ao escrever no log");
    } else {
        eprintln!(
            "Erro ao restaurar o backup: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

pub fn list_backups(tool_path: &Path) {
    let entries = fs::read_dir(tool_path).expect("Falha ao ler o diretório de backups");

    let mut backups = vec![];
    let last_restored = read_last_restored();

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("sql")) {
                let metadata = fs::metadata(&path).expect("Erro ao obter metadados do arquivo");
                let created = metadata.created().expect("Erro ao obter data de criação");

                if let Ok(duration_since_epoch) = created.duration_since(SystemTime::UNIX_EPOCH) {
                    let created_naive = NaiveDateTime::from_timestamp(
                        duration_since_epoch.as_secs() as i64,
                        duration_since_epoch.subsec_nanos(),
                    );

                    let formatted_time = created_naive.format("%d/%m/%Y %H:%M").to_string();

                    if let Some(file_name) = path.file_name() {
                        if let Some(file_str) = file_name.to_str() {
                            let is_last_restored = if let Some(last_backup) = &last_restored {
                                file_str == last_backup
                            } else {
                                false
                            };
                            backups.push((file_str.to_string(), formatted_time, is_last_restored));
                        }
                    }
                } else {
                    eprintln!(
                        "Erro ao calcular a duração desde a época para o arquivo: {}",
                        path.display()
                    );
                }
            }
        }
    }

    println!("======================================================================");
    println!(
        "{:<27} | {:<20} | {:<20}",
        "   ARQUIVOS", "   DATA", "ÚLTIMO RESTAURADO"
    );
    println!("======================================================================");

    for (file, time, is_last_restored) in backups {
        let last_restored_label = if is_last_restored { "SIM" } else { "NÃO" };
        println!("{:<20} | {:<20} | {:<20}", file, time, last_restored_label);
    }

    println!("======================================================================");
}

fn read_last_restored() -> Option<String> {
    let log_file_path = get_bkp_path().join(".purrs");

    if let Ok(contents) = fs::read_to_string(log_file_path) {
        let last_line = contents.lines().last();

        if let Some(last_entry) = last_line {
            let parts: Vec<&str> = last_entry.split_whitespace().collect();
            if parts.len() >= 4 {
                return Some(parts[3].to_string());
            }
        }
    }

    println!("Foi impossível abrir o arquivo dos purrs");

    None
}
