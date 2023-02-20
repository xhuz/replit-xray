use std::{
    fs::File,
    io::{Cursor, Read, Write},
};

use anyhow::Result;
use cmd_lib::{run_cmd, run_fun};
use regex::Regex;
use reqwest::blocking::Client;
use serde_yaml;
use uuid::Uuid;
use zip::ZipArchive;

use crate::config::{app::Config, server::ServerConfig};

#[derive(Debug, PartialEq, Eq)]
struct Version {
    pub inner: String,
}

#[derive(Debug)]
pub struct Server {
    config: Config,
    version: Version,
    latest_version: Version,
}

impl<T> From<T> for Version
where
    T: ?Sized + Into<String>,
{
    fn from(value: T) -> Self {
        Version {
            inner: value.into(),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Version {
            inner: "".to_owned(),
        }
    }
}

impl Version {
    fn empty(&self) -> bool {
        &self.inner == ""
    }
}

impl Server {
    pub fn new() -> Self {
        let mut s = Server {
            config: Config::from_env(),
            version: Version::default(),
            latest_version: Version::default(),
        };

        s.version = s.get_current_version();

        s.latest_version = s.get_latest_version();

        s
    }

    pub fn run(&self) -> Result<()> {
        if self.version != self.latest_version {
            if self.download().is_err() && self.version.empty() {
                panic!("download failed")
            }
        }

        let uuid = self.uuid();

        let c = ServerConfig::new(&uuid, &uuid);

        let yaml = serde_yaml::to_string(&c)?;

        File::create("./config.yml")?.write_all(&yaml.as_bytes())?;

        let cmd = format!(
            "{} -c config.yml",
            &self.config.bin_path().to_string_lossy()
        );

        let path = &self.config.bin_path();

        run_cmd!(chmod "+x" $path)?;

        run_cmd!($cmd)?;

        self.share(&uuid);

        Ok(())
    }

    fn share(&self, uuid: &str) {
        let share_url = format!(
            "{}.{}.replit.co/{}",
            self.config.repl_slug, self.config.repl_owner, uuid
        );

        println!("{share_url}")
    }

    fn get_uuid_from_replit_db(&self) -> Result<String> {
        let res = Client::new()
            .get(format!("{}/uuid", &self.config.replit_db_url))
            .send()?;

        let t = res.text()?;

        Ok(t)
    }

    fn set_uuid_to_replit_db(&self, uuid: &Uuid) -> Result<()> {
        Client::new()
            .post(format!("{}/uuid={}", &self.config.replit_db_url, uuid))
            .send()?;

        Ok(())
    }

    fn uuid(&self) -> String {
        let uuid = match self.get_uuid_from_replit_db() {
            Ok(u) => u,
            Err(_) => {
                let u = Uuid::new_v4();
                self.set_uuid_to_replit_db(&u)
                    .expect_err("write uuid to db failed");
                u.to_string()
            }
        };

        uuid
    }

    fn get_current_version(&self) -> Version {
        let p = self.config.bin_path();
        let result = run_fun!($p "--version").unwrap_or_default();

        Version { inner: result }
    }

    fn get_latest_version(&self) -> Version {
        let url = self.config.remote_addr("latest");
        let res = Client::new()
            .head(url)
            .send()
            .expect("get latest version failed");

        let reg = Regex::new(r"[^/]+$").unwrap();

        let final_url = res.url().to_string();

        println!("{final_url}");

        match reg.find(&res.url().to_string()) {
            Some(v) => Version {
                inner: v.as_str().to_owned(),
            },
            None => Version::default(),
        }
    }

    fn unzip(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let c = Cursor::new(bytes);

        let mut buf = vec![];

        ZipArchive::new(c)?
            .by_name(&self.config.name)?
            .read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn download(&self) -> Result<()> {
        let url = self.config.remote_addr(&format!(
            "{}/download/{}/Xray-{}-64.zip",
            self.config.base_remote_addr,
            self.latest_version.inner,
            std::env::consts::OS
        ));

        let res = Client::new().get(url).send()?;

        let bytes = res.bytes()?;

        let buf = self.unzip(&bytes)?;

        File::create(&self.config.bin_path())?.write_all(&buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use regex::Regex;

    #[test]
    fn test_match_version() {
        let reg = Regex::new(r"[^/]+$").unwrap();

        let url = "https://github.com/XTLS/Xray-core/releases/tag/v1.7.5";

        let matched = reg.find(url).and_then(|m| Some(m.as_str()));

        assert_eq!(Some("v1.7.5"), matched);
    }
}
