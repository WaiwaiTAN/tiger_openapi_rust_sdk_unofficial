use crate::{constants, properties::Properties};
use rand::Rng;
use secrecy::{ExposeSecret, SecretString};
use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("credential directory was not explicitly configured")]
    MissingDirectory,
    #[error("credential path is not a directory: {0}")]
    NotDirectory(PathBuf),
    #[error("required credential file is missing: {0}")]
    MissingFile(PathBuf),
    #[error("credential file permissions are too broad: {0}")]
    UnsafePermissions(PathBuf),
    #[error("failed to access credential file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("malformed properties file {0}")]
    Malformed(PathBuf),
    #[error("required configuration field is missing: {0}")]
    MissingField(&'static str),
}

pub fn get_device_id() -> String {
    let mut b = [0_u8; 6];
    rand::rng().fill(&mut b);
    b.iter()
        .map(|v| format!("{v:02x}"))
        .collect::<Vec<_>>()
        .join(":")
}

#[derive(Clone)]
pub struct ClientConfig {
    pub tiger_id: SecretString,
    pub private_key: SecretString,
    pub secret_key: SecretString,
    pub charset: String,
    pub version: String,
    pub sign_type: String,
    pub device_id: String,
    pub account: SecretString,
    pub license: String,
    pub token: SecretString,
    pub lang: String,
    pub sandbox_debug: bool,
    pub server_url: String,
    pub server_public_key: String,
    pub socket_url: String,
    pub socket_port: String,
    pub props_path: Option<PathBuf>,
}
impl Default for ClientConfig {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientConfig")
            .field("tiger_id", &"[REDACTED]")
            .field("private_key", &"[REDACTED]")
            .field("secret_key", &"[REDACTED]")
            .field("account", &"[REDACTED]")
            .field("token", &"[REDACTED]")
            .field("charset", &self.charset)
            .field("sign_type", &self.sign_type)
            .field("sandbox_debug", &self.sandbox_debug)
            .field("server_url", &self.server_url)
            .finish_non_exhaustive()
    }
}

impl ClientConfig {
    pub fn new() -> Self {
        Self {
            tiger_id: "".into(),
            private_key: "".into(),
            secret_key: "".into(),
            charset: constants::DEFAULT_CHAR_SET.into(),
            version: constants::OPEN_API_SERVICE_VERSION.into(),
            sign_type: constants::DEFAULT_SIGN_TYPE.into(),
            device_id: get_device_id(),
            account: "".into(),
            license: String::new(),
            token: "".into(),
            lang: "en_US".into(),
            sandbox_debug: false,
            server_url: constants::TIGER_SERVER_URL.into(),
            server_public_key: constants::TIGER_PUBLIC_KEY.into(),
            socket_url: constants::TIGER_SOCKET_HOST.into(),
            socket_port: constants::TIGER_SOCKET_PORT.into(),
            props_path: None,
        }
    }
    pub fn with_credential_directory(path: impl AsRef<Path>) -> Self {
        let mut c = Self::new();
        c.props_path = Some(path.as_ref().into());
        c
    }
    pub fn get_props_path(&self, name: &str) -> Result<PathBuf, ConfigError> {
        let root = self
            .props_path
            .as_ref()
            .ok_or(ConfigError::MissingDirectory)?;
        if root.is_file() {
            if name == constants::DEFAULT_PROPS_FILE {
                return Ok(root.clone());
            }
            return root
                .parent()
                .map(|parent| parent.join(name))
                .ok_or_else(|| ConfigError::NotDirectory(root.clone()));
        }
        if !root.is_dir() {
            return Err(ConfigError::NotDirectory(root.clone()));
        }
        Ok(root.join(name))
    }
    pub fn get_token_path(&self) -> Result<PathBuf, ConfigError> {
        self.get_props_path(constants::DEFAULT_TOKEN_FILE)
    }
    fn validate_file(path: &Path) -> Result<(), ConfigError> {
        if !path.is_file() {
            return Err(ConfigError::MissingFile(path.into()));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(path)
                .map_err(|source| ConfigError::Io {
                    path: path.into(),
                    source,
                })?
                .permissions()
                .mode()
                & 0o077
                != 0
            {
                return Err(ConfigError::UnsafePermissions(path.into()));
            }
        }
        Ok(())
    }
    fn read_properties(path: &Path) -> Result<Properties, ConfigError> {
        Self::validate_file(path)?;
        let file = File::open(path).map_err(|source| ConfigError::Io {
            path: path.into(),
            source,
        })?;
        let mut p = Properties::default();
        p.load_strict(BufReader::new(file))
            .map_err(|_| ConfigError::Malformed(path.into()))?;
        Ok(p)
    }
    pub fn load_props(&mut self) -> Result<(), ConfigError> {
        let path = self.get_props_path(constants::DEFAULT_PROPS_FILE)?;
        let p = Self::read_properties(&path)?;
        if self.tiger_id.expose_secret().is_empty()
            && let Some(v) = p.get_property("tiger_id")
        {
            self.tiger_id = v.into();
        }
        if self.private_key.expose_secret().is_empty()
            && let Some(v) = p.get_property("private_key_pk1")
        {
            self.private_key = v.into();
        }
        if self.account.expose_secret().is_empty()
            && let Some(v) = p.get_property("account")
        {
            self.account = v.into();
        }
        if self.license.is_empty() {
            self.license = p.get_property("license").unwrap_or_default();
        }
        self.sandbox_debug = p
            .get_property("env")
            .is_some_and(|v| v.eq_ignore_ascii_case("sandbox"));
        if self.is_us() {
            self.server_url = constants::US_TIGER_SERVER_URL.into();
            self.socket_url = constants::US_TIGER_SOCKET_HOST.into();
            self.socket_port = constants::US_TIGER_SOCKET_PORT.into();
        }
        self.validate_quote()
    }
    pub fn load_token(&mut self) -> Result<(), ConfigError> {
        let path = self.get_token_path()?;
        let p = Self::read_properties(&path)?;
        self.token = p
            .get_property("token")
            .ok_or(ConfigError::MissingField("token"))?
            .into();
        Ok(())
    }
    pub fn save_token(&mut self, new_token: impl Into<SecretString>) -> Result<(), ConfigError> {
        let new_token = new_token.into();
        let path = self.get_token_path()?;
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|source| ConfigError::Io {
                path: path.clone(),
                source,
            })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|source| {
                ConfigError::Io {
                    path: path.clone(),
                    source,
                }
            })?;
        }
        let mut p = Properties::default();
        p.set_property("token", new_token.expose_secret());
        p.store(BufWriter::new(file))
            .map_err(|source| ConfigError::Io { path, source })?;
        self.token = new_token;
        Ok(())
    }
    pub fn is_us(&self) -> bool {
        self.license == "TBUS"
    }
    pub fn validate_quote(&self) -> Result<(), ConfigError> {
        if self.tiger_id.expose_secret().is_empty() {
            Err(ConfigError::MissingField("tiger_id"))
        } else if self.private_key.expose_secret().is_empty() {
            Err(ConfigError::MissingField("private_key_pk1"))
        } else {
            Ok(())
        }
    }
    pub fn check(&self) -> Result<(), ConfigError> {
        self.validate_quote()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[test]
    fn debug_is_redacted() {
        let mut c = ClientConfig::new();
        c.tiger_id = "identifier-canary".into();
        c.private_key = "key-canary".into();
        c.token = "token-canary".into();
        let d = format!("{c:?}");
        assert!(!d.contains("canary"));
        assert!(d.contains("REDACTED"));
    }
    #[test]
    fn directory_is_never_implicitly_discovered() {
        assert!(matches!(
            ClientConfig::new().load_props(),
            Err(ConfigError::MissingDirectory)
        ));
    }
    #[test]
    fn missing_and_malformed_files_are_typed_errors() {
        let dir = tempdir().unwrap();
        let mut c = ClientConfig::with_credential_directory(dir.path());
        assert!(matches!(c.load_props(), Err(ConfigError::MissingFile(_))));
        let path = dir.path().join(constants::DEFAULT_PROPS_FILE);
        fs::write(&path, "not a property line\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert!(matches!(c.load_props(), Err(ConfigError::Malformed(_))));
    }
    #[cfg(unix)]
    #[test]
    fn broad_permissions_are_rejected() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let path = dir.path().join(constants::DEFAULT_PROPS_FILE);
        fs::write(&path, "tiger_id=test\nprivate_key_pk1=test\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let mut c = ClientConfig::with_credential_directory(dir.path());
        assert!(matches!(
            c.load_props(),
            Err(ConfigError::UnsafePermissions(_))
        ));
    }
}
