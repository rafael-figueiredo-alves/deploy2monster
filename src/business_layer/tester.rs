use crate::entities::project::Project;
use crate::shared::db_errors::friendly_mysql_error;
use crate::shared::ftp_errors::friendly_ftp_error;
use mysql::{OptsBuilder, Pool};
use suppaftp::FtpStream;

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

    crate::shared::message_functions::write_info(&format!("Conectando em {}...", host_port));

    let mut stream = match FtpStream::connect(&host_port) {
        Ok(s) => {
            crate::shared::message_functions::write_success("Servidor FTP alcançado.");
            s
        }
        Err(e) => {
            crate::shared::message_functions::write_error(&friendly_ftp_error(
                &e.to_string(),
                &host_port,
            ));
            return;
        }
    };

    match stream.login(&ftp.ftp_user, &ftp.ftp_password) {
        Ok(_) => crate::shared::message_functions::write_success("Autenticação FTP bem-sucedida."),
        Err(e) => {
            crate::shared::message_functions::write_error(&friendly_ftp_error(
                &e.to_string(),
                &host_port,
            ));
            stream.quit().ok();
            return;
        }
    }

    match stream.pwd() {
        Ok(dir) => crate::shared::message_functions::write_info(&format!("Pasta remota atual: {}", dir)),
        Err(e) => crate::shared::message_functions::write_warning(&friendly_ftp_error(
            &e.to_string(),
            &host_port,
        )),
    }

    match stream.nlst(Some("/wwwroot")) {
        Ok(files) => crate::shared::message_functions::write_success(&format!(
            "/wwwroot acessível — {} arquivo(s) encontrado(s).",
            files.len()
        )),
        Err(e) => crate::shared::message_functions::write_warning(&friendly_ftp_error(
            &e.to_string(),
            "/wwwroot",
        )),
    }

    stream.quit().ok();
    crate::shared::message_functions::write_success("Conexão FTP encerrada com sucesso.");
}

fn test_database(project: &Project) {
    println!("  — Banco de Dados —");

    let db = &project.database_settings;

    crate::shared::message_functions::write_info(&format!(
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
        Ok(p) => p,
        Err(e) => {
            crate::shared::message_functions::write_error(&friendly_mysql_error(
                &e,
                &db.host,
                db.port,
                &db.database,
                &db.user,
            ));
            return;
        }
    };

    let mut conn = match pool.get_conn() {
        Ok(c) => {
            crate::shared::message_functions::write_success("Conexão com banco estabelecida.");
            c
        }
        Err(e) => {
            crate::shared::message_functions::write_error(&friendly_mysql_error(
                &e,
                &db.host,
                db.port,
                &db.database,
                &db.user,
            ));
            return;
        }
    };

    // testa com query simples
    use mysql::prelude::Queryable;
    match conn.query_drop("SELECT 1") {
        Ok(_) => crate::shared::message_functions::write_success("Query de teste executada com sucesso."),
        Err(e) => crate::shared::message_functions::write_error(&friendly_mysql_error(
            &e,
            &db.host,
            db.port,
            &db.database,
            &db.user,
        )),
    }

    // verifica quantidade de tabelas
    match conn.query_first::<u64, _>("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE()")
    {
        Ok(Some(count)) => {
            crate::shared::message_functions::write_info(&format!("Banco possui {} tabela(s).", count))
        }
        _ => {}
    }
}
