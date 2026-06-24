use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FtpSettings {
    pub ftp_host: String,
    pub ftp_port: u16,
    pub ftp_user: String,
    pub ftp_password: String,    
}

// region: Builder

#[derive(Default)]
pub struct FtpSettingsBuilder {
    ftp_host: Option<String>,
    ftp_port: Option<u16>,
    ftp_user: Option<String>,
    ftp_password: Option<String>,
}

impl FtpSettingsBuilder {
    pub fn ftp_host(mut self, host: impl Into<String>) -> Self {
        self.ftp_host = Some(host.into());
        self
    }

    pub fn ftp_port(mut self, port: u16) -> Self {
        self.ftp_port = Some(port);
        self
    }

    pub fn ftp_user(mut self, user: impl Into<String>) -> Self {
        self.ftp_user = Some(user.into());
        self
    }

    pub fn ftp_password(mut self, password: impl Into<String>) -> Self {
        self.ftp_password = Some(password.into());
        self
    }

    pub fn build(self) -> Result<FtpSettings, String> {
        Ok(FtpSettings {
            ftp_host: self.ftp_host.ok_or("ftp_host é obrigatório")?,
            ftp_port: self.ftp_port.ok_or("ftp_port é obrigatório")?,
            ftp_user: self.ftp_user.ok_or("ftp_user é obrigatório")?,
            ftp_password: self.ftp_password.ok_or("ftp_password é obrigatório")?,
        })
    }
}

impl FtpSettings {
    pub fn builder() -> FtpSettingsBuilder {
        FtpSettingsBuilder::default()
    }
}

// endregion

// region: CONTEXT para ser usado dentro do ProjectBuilder

pub trait FtpSettingsParent {
    fn set_ftp_settings(&mut self, settings: FtpSettings);
}

pub struct FtpSettingsContext<P> {
    parent: P,
    builder: FtpSettingsBuilder,
}

impl<P> FtpSettingsContext<P>
where
    P: FtpSettingsParent,
{
    pub fn ftp_host(mut self, host: impl Into<String>) -> Self {
        self.builder = self.builder.ftp_host(host);
        self
    }

    pub fn ftp_port(mut self, port: u16) -> Self {
        self.builder = self.builder.ftp_port(port);
        self
    }

    pub fn ftp_user(mut self, user: impl Into<String>) -> Self {
        self.builder = self.builder.ftp_user(user);
        self
    }

    pub fn ftp_password(mut self, password: impl Into<String>) -> Self {
        self.builder = self.builder.ftp_password(password);
        self
    }

    pub fn end(self) -> P {
        let ftp = self.builder.build().expect("Failed to build FtpSettings");
        let mut parent = self.parent;
        parent.set_ftp_settings(ftp);
        parent
    }
}

// endregion