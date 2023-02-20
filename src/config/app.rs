use std::{collections::HashMap, env, io, path::PathBuf};

#[derive(Debug)]
pub(crate) struct Config {
    pub replit_db_url: String,
    pub repl_slug: String,
    pub repl_owner: String,
    pub name: String,
    pub fake_name: String,
    pub base_remote_addr: String,
}

impl Config {
    pub fn from_env() -> Self {
        let env_vars: HashMap<String, String> = env::vars().collect();
        if let (Some(db), Some(slug), Some(owner)) = (
            env_vars.get("REPLIT_DB_URL"),
            env_vars.get("REPL_SLUG"),
            env_vars.get("REPL_OWNER"),
        ) {
            Config {
                replit_db_url: db.to_owned(),
                repl_slug: slug.to_owned(),
                repl_owner: owner.to_owned(),
                name: "xray".to_owned(),
                fake_name: "server".to_owned(),
                base_remote_addr: "https://github.com/XTLS/Xray-core/releases".to_owned(),
            }
        } else {
            panic!("not found env")
        }
    }

    pub fn bin_path(&self) -> PathBuf {
        PathBuf::from(format!("./{}", self.fake_name))
    }

    pub fn remote_addr(&self, path: &str) -> String {
        let p = PathBuf::from(&self.base_remote_addr).join(path);

        match p.as_path().to_str() {
            Some(a) => a.to_owned(),
            None => panic!("remote addr invalid"),
        }
    }
}
