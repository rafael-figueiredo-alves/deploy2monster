use serde::{Serialize, Deserialize};

use super::database_settings::{
    DatabaseSettings,
    DatabaseSettingsBuilder,
    DatabaseSettingsContext,
    DatabaseSettingsParent,
};
use super::ftp_settings::{
    FtpSettings,
    FtpSettingsBuilder,
    FtpSettingsContext,
    FtpSettingsParent,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub name: String,
    pub publish_folder: String,
    pub project_file:   String,
    pub ftp_settings: FtpSettings,
    pub database_settings: DatabaseSettings,
    pub sql_script: String,
}

// region: Builder

#[derive(Default)]
pub struct ProjectBuilder {
    name: Option<String>,
    publish_folder: Option<String>,
    project_file: Option<String>,
    ftp_settings: Option<FtpSettings>,
    database_settings: Option<DatabaseSettings>,
    sql_script: Option<String>,
}

impl ProjectBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn publish_folder(mut self, folder: impl Into<String>) -> Self {
        self.publish_folder = Some(folder.into());
        self
    }

    pub fn project_file(mut self, file: impl Into<String>) -> Self {
        self.project_file = Some(file.into());
        self
    }

    pub fn sql_script(mut self, script: impl Into<String>) -> Self {
        self.sql_script = Some(script.into());
        self
    }

    pub fn database_settings(self) -> DatabaseSettingsContext<Self> {
        DatabaseSettingsContext {
            parent: self,
            builder: DatabaseSettingsBuilder::default(),
        }
    }

    pub fn ftp_settings(self) -> FtpSettingsContext<Self> {
        FtpSettingsContext {
            parent: self,
            builder: FtpSettingsBuilder::default(),
        }
    }

    pub fn build(self) -> Result<Project, String> {
        Ok(Project {
            name: self.name.ok_or("name é obrigatório")?,
            publish_folder: self.publish_folder.ok_or("publish_folder é obrigatório")?,
            project_file: self.project_file.ok_or("project_file é obrigatório")?,
            ftp_settings: self.ftp_settings.ok_or("ftp_settings é obrigatório")?,
            database_settings: self.database_settings.ok_or("database_settings é obrigatório")?,
            sql_script: self.sql_script.unwrap_or_default(),
        })
    }
}

impl DatabaseSettingsParent for ProjectBuilder {
    fn set_database_settings(&mut self, settings: DatabaseSettings) {
        self.database_settings = Some(settings);
    }
}

impl FtpSettingsParent for ProjectBuilder {
    fn set_ftp_settings(&mut self, settings: FtpSettings) {
        self.ftp_settings = Some(settings);
    }
}

impl Project {
    pub fn builder() -> ProjectBuilder {
        ProjectBuilder::default()
    }
}

// endregion