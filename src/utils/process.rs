use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

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

        // Tentar fazer o parsing
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
