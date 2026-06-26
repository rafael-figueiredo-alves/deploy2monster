use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

// region: Builder

#[derive(Default)]
pub struct DatabaseSettingsBuilder {
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    database: Option<String>,
}

impl DatabaseSettingsBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn build(self) -> Result<DatabaseSettings, String> {
        Ok(DatabaseSettings {
            host: self.host.ok_or("host é obrigatório")?,
            port: self.port.ok_or("port é obrigatório")?,
            user: self.user.ok_or("user é obrigatório")?,
            password: self.password.ok_or("password é obrigatório")?,
            database: self.database.ok_or("database é obrigatório")?,
        })
    }
}

impl DatabaseSettings {
/*     pub fn builder() -> DatabaseSettingsBuilder {
        DatabaseSettingsBuilder::default()
    } */
}

// endregion

// region: Context para ser usado dentro do ProjectBuilder

pub trait DatabaseSettingsParent {
    fn set_database_settings(&mut self, settings: DatabaseSettings);
}

pub struct DatabaseSettingsContext<P> {
    pub parent: P,
    pub builder: DatabaseSettingsBuilder,
}

impl<P> DatabaseSettingsContext<P>
where
    P: DatabaseSettingsParent,
{
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.builder = self.builder.host(host);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.builder = self.builder.port(port);
        self
    }

    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.builder = self.builder.user(user);
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.builder = self.builder.password(password);
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.builder = self.builder.database(database);
        self
    }

    pub fn end(self) -> P {
        let db = self.builder.build().expect("Falhou em construir DatabaseSettings");
        let mut parent = self.parent;
        parent.set_database_settings(db);
        parent
    }
}

// endregion