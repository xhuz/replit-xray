use std::{
    env::consts::OS,
    error, fmt,
    fs::{self, File, Permissions},
    io::{self, Cursor, Read, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    string::FromUtf8Error,
};

use reqwest::{blocking::Client, Error as RequestError};

use zip::{result::ZipError, ZipArchive};

#[derive(Debug)]
enum Error {
    IOError(io::Error),
    ParseError(FromUtf8Error),
    HttpClientError(RequestError),
    VersionError(String),
    ZipError(ZipError),
    FilenameError,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IOError(value)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::ParseError(value)
    }
}

impl From<RequestError> for Error {
    fn from(value: RequestError) -> Self {
        Error::HttpClientError(value)
    }
}

impl From<ZipError> for Error {
    fn from(value: ZipError) -> Self {
        Error::ZipError(value)
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(e) => write!(f, "IO Error: {}", e.to_string()),
            Error::ParseError(e) => write!(f, "Parse String Error: {}", e.to_string()),
            Error::HttpClientError(e) => write!(f, "Http Request Error: {}", e.to_string()),
            Error::VersionError(e) => write!(f, "Parse Remote Version Error: {}", e),
            Error::ZipError(e) => write!(f, "Unzip Error: {}", e.to_string()),
            Error::FilenameError => write!(f, "Invalid Filename"),
        }
    }
}

struct Xray {
    remote_addr: &'static str,
    bin_path: &'static str,
}

impl Xray {
    fn remote_addr(&self, path: &str) -> String {
        let mut url = self.remote_addr.to_owned();

        if !url.ends_with("/") {
            url.push_str("/");
        }

        let mut s: Option<&str> = None;

        if path.starts_with("/") {
            s = Some(&path[1..])
        }

        url.push_str(s.unwrap_or(path));

        url
    }

    fn get_current_version(&self) -> Result<String, Error> {
        self.set_mode(0o755)?;

        let out = Command::new(self.bin_path).arg("--version").output()?;

        if out.status.success() {
            let result = out.stdout;
            let info = String::from_utf8(result)?;

            let s = info.split(" ").collect::<Vec<&str>>();

            let mut v = "v".to_string();

            v.push_str(s[1]);

            Ok(v)
        } else {
            Err(Error::VersionError(String::from_utf8(out.stderr)?))
        }
    }

    fn get_latest_version(&self) -> Result<String, Error> {
        let res = Client::new().get(self.remote_addr("latest")).send()?;

        let url = res.url().as_str();

        match url.split("/").collect::<Vec<&str>>().pop() {
            Some(v) => Ok(v.to_owned()),
            None => Err(Error::VersionError(url.to_owned())),
        }
    }

    fn set_mode(&self, mode: u32) -> Result<(), Error> {
        let permissions = Permissions::from_mode(mode);
        fs::set_permissions(self.bin_path, permissions)?;
        Ok(())
    }

    fn unzip(&self, bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let c = Cursor::new(bytes);

        let mut buf = vec![];

        let filename = Path::new(self.bin_path)
            .file_name()
            .map(|f| f.to_str().ok_or(Error::FilenameError))
            .ok_or(Error::FilenameError)??;

        ZipArchive::new(c)?
            .by_name(filename)?
            .read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn download(&self, version: &str) -> Result<(), Error> {
        let url = self.remote_addr(&format!("download/{}/Xray-{}-64.zip", version, OS));

        let bytes = Client::new().get(url).send()?.bytes()?;

        let buf = self.unzip(&bytes)?;

        let mut f = File::create(self.bin_path)?;

        f.write_all(&buf)?;

        self.set_mode(0o755)?;

        Ok(())
    }
}

const XRAY: Xray = Xray {
    remote_addr: "https://github.com/XTLS/Xray-core/releases",
    bin_path: "bin/xray",
};

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    let current_version = XRAY.get_current_version();

    let latest_version = XRAY.get_latest_version();

    if let (Ok(cur), Ok(latest)) = (&current_version, &latest_version) {
        if cur != latest {
            XRAY.download(&latest).unwrap();
        }
    } else if let Ok(v) = latest_version {
        XRAY.download(&v).unwrap();
    } else if let (Err(a), Err(b)) = (current_version, latest_version) {
        panic!("Build Error, {}, {}", a.to_string(), b.to_string())
    }
}
