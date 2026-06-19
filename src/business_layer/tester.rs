use crate::projects::Project;
use suppaftp::FtpStream;
use mysql::{Pool, OptsBuilder};

pub fn run(project: &Project) {
    println!();
    println!("  Testando conexões do projeto '{}'...", project.name);
    println!();

    test_ftp(project);
    println!();
    test_database(project);
    println!();
}

fn test_ftp(project: &Project) {
    println!("  — FTP —");

    let ftp = &project.ftp_settings;
    let host_port = format!("{}:{}", ftp.ftp_host, ftp.ftp_port);

    crate::ui::write_info(&format!("Conectando em {}...", host_port));

    let mut stream = match FtpStream::connect(&host_port) {
        Ok(s)  => {
            crate::ui::write_success("Servidor FTP alcançado.");
            s
        }
        Err(e) => {
            crate::ui::write_error(&format!("Falha ao conectar: {}", e));
            return;
        }
    };

    match stream.login(&ftp.ftp_user, &ftp.ftp_password) {
        Ok(_)  => crate::ui::write_success("Autenticação FTP bem-sucedida."),
        Err(e) => {
            crate::ui::write_error(&format!("Falha na autenticação: {}", e));
            stream.quit().ok();
            return;
        }
    }

    match stream.pwd() {
        Ok(dir) => crate::ui::write_info(&format!("Pasta remota atual: {}", dir)),
        Err(e)  => crate::ui::write_warning(&format!("Não foi possível obter pasta remota: {}", e)),
    }

    match stream.nlst(Some("/wwwroot")) {
        Ok(files) => crate::ui::write_success(&format!(
            "/wwwroot acessível — {} arquivo(s) encontrado(s).",
            files.len()
        )),
        Err(e) => crate::ui::write_warning(&format!("/wwwroot não acessível: {}", e)),
    }

    stream.quit().ok();
    crate::ui::write_success("Conexão FTP encerrada com sucesso.");
}

fn test_database(project: &Project) {
    println!("  — Banco de Dados —");

    let db = &project.database_settings;

    crate::ui::write_info(&format!(
        "Conectando em {}:{}/{}...",
        db.host, db.port, db.database
    ));

    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(&db.host))
        .tcp_port(db.port)
        .user(Some(&db.user))
        .pass(Some(&db.password))
        .db_name(Some(&db.database));

    let pool = match Pool::new(opts) {
        Ok(p)  => p,
        Err(e) => {
            crate::ui::write_error(&format!("Falha ao conectar: {}", e));
            return;
        }
    };

    let mut conn = match pool.get_conn() {
        Ok(c)  => {
            crate::ui::write_success("Conexão com banco estabelecida.");
            c
        }
        Err(e) => {
            crate::ui::write_error(&format!("Falha ao obter conexão: {}", e));
            return;
        }
    };

    // testa com query simples
    use mysql::prelude::Queryable;
    match conn.query_drop("SELECT 1") {
        Ok(_)  => crate::ui::write_success("Query de teste executada com sucesso."),
        Err(e) => crate::ui::write_error(&format!("Falha na query de teste: {}", e)),
    }

    // verifica quantidade de tabelas
    match conn.query_first::<u64, _>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE()"
    ) {
        Ok(Some(count)) => crate::ui::write_info(&format!("Banco possui {} tabela(s).", count)),
        _ => {}
    }
}