use std::{collections::HashMap, env, path::PathBuf};

#[derive(Debug)]
pub(crate) struct Config {
    pub replit_db_url: String,
    pub repl_slug: String,
    pub repl_owner: String,
    pub bin_name: String,
    pub asset_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        let env_vars: HashMap<String, String> = env::vars().collect();
        if let (Some(db), Some(slug), Some(owner)) = (
            env_vars.get("REPLIT_DB_URL").or(Some(&"".to_owned())),
            env_vars.get("REPL_SLUG").or(Some(&"".to_owned())),
            env_vars.get("REPL_OWNER").or(Some(&"".to_owned())),
        ) {
            Config {
                replit_db_url: db.to_owned(),
                repl_slug: slug.to_owned(),
                repl_owner: owner.to_owned(),
                bin_name: "x".to_owned(),
                asset_name: "xray".to_owned(),
            }
        } else {
            panic!("not found env")
        }
    }

    pub fn bin_path(&self) -> PathBuf {
        PathBuf::from(format!("./{}", self.bin_name))
    }
}
