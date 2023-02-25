use std::{
    fs::{self, File, Permissions},
    io::Write,
    os::unix::prelude::PermissionsExt,
    process::{Child, Command, Stdio},
};

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use rust_embed::RustEmbed;
use uuid::Uuid;

use crate::config::{app::Config, server::ServerConfig};

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/bin"]
struct Assets;

#[derive(Debug, PartialEq, Eq)]
struct Version {
    pub inner: String,
}

#[derive(Debug)]
pub struct Server {
    config: Config,
    version: Version,
    child_process: Option<Child>,
    keep_alive: Option<String>,
    uuid: Option<String>,
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

impl Server {
    pub fn new() -> Self {
        let s = Server {
            config: Config::from_env(),
            version: Version::default(),
            child_process: None,
            keep_alive: None,
            uuid: None,
        };

        s
    }

    pub fn run(&mut self) -> Result<()> {
        self.prepare()?;

        let uuid = self.uuid();

        let c = ServerConfig::new(&uuid, &uuid);

        println!("{:#?}", &c);

        let yaml = serde_yaml::to_string(&c)?;

        File::create("./config.yml")?.write_all(&yaml.as_bytes())?;

        let child = Command::new(&self.config.bin_path())
            .args(["-c", "config.yml"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        self.child_process = Some(child);

        self.keep_alive = Some(format!(
            "{}.{}.replit.co/{}",
            self.config.repl_slug, self.config.repl_owner, uuid
        ));

        self.share(&uuid);

        Ok(())
    }

    pub fn keep(&self) {
        if let Some(url) = &self.keep_alive {
            if let Ok(res) = Client::new().get(url).send() {
                if let Ok(t) = res.text() {
                    println!("{t}")
                }
            } else {
                println!("Bad Request")
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(c) = &mut self.child_process {
            c.kill().expect("command wasn't running");
        }
    }

    fn prepare(&mut self) -> Result<()> {
        let embed = Assets::get(&self.config.asset_name).ok_or(anyhow!("Get Embed File Failed"))?;

        let mut f = File::create(&self.config.bin_path())?;

        f.write_all(&embed.data.to_vec())?;

        let p = Permissions::from_mode(0o755);

        fs::set_permissions(&self.config.bin_path(), p)?;

        drop(f);

        self.version = self.get_version();

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
            .send()?
            .error_for_status()?;

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

    fn get_version(&self) -> Version {
        let p = self.config.bin_path();

        let out = Command::new(&p)
            .arg("--version")
            .output()
            .expect(&format!("exec command failed, {:?}", &p));

        let result = String::from_utf8(out.stdout).unwrap();

        let a = result.split(" ").collect::<Vec<&str>>()[1];

        Version {
            inner: format!("v{a}"),
        }
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

    #[test]
    fn test_match_version_from_bin() {
        let text = r"Xray 1.7.5 (Xray, Penetrates Everything.) Custom (go1.20 linux/amd64)
        A unified platform for anti-censorship.";

        let v: Vec<&str> = text.split(" ").collect();

        assert_eq!("1.7.5", v[1]);
    }
}
