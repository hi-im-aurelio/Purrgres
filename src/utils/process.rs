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
            clear_pid();
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
