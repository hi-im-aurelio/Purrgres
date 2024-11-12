use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(name = "purrgres")]
#[command(about = "Uma ferramenta de backup automática para PostgreSQL em contêineres Docker.")]
pub struct Args {
    #[arg(short, long, help = "Usuário do PostgreSQL", required_unless_present_any = ["stats", "stop", "list_purrs"])]
    pub user: Option<String>,

    #[arg(short, long, help = "Nome do banco de dados", required_unless_present_any = ["stats", "stop", "list_purrs"])]
    pub database: Option<String>,

    #[arg(short, long, help = "Nome do container", required_unless_present_any = ["stats", "stop", "list_purrs"])]
    pub container: Option<String>,

    #[arg(long, help = "Verifica o status do backup em execução.")]
    pub stats: bool,

    #[arg(long, help = "Para o processo de backup")]
    pub stop: bool,

    #[arg(long, help = "Lista os backups realizados")]
    pub list_purrs: bool,

    #[arg(long, help = "Aplica um backup a partir de um arquivo")]
    pub rpurry: Option<String>,
}
