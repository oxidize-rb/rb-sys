use std::{
    env,
    error::Error,
    path::PathBuf,
    process::{Command, Output},
};

#[derive(Debug, Clone)]
pub struct Build {
    ruby_version: Option<String>,
    build_dir: Option<PathBuf>,
    sha256: Option<&'static str>,
    prefix: Option<PathBuf>,
}

impl Default for Build {
    fn default() -> Self {
        let build_dir = env::var("OUT_DIR")
            .ok()
            .map(|s| PathBuf::from(s).join("rb-src-build"));

        let prefix = env::var("OUT_DIR")
            .ok()
            .map(|s| PathBuf::from(s).join("ruby"));

        Self {
            prefix,
            build_dir,
            sha256: None,
            ruby_version: None,
        }
    }
}

impl Build {
    /// Sets the Ruby version to build.
    pub fn ruby_version<T: Into<String>>(&mut self, version: T) -> &mut Self {
        self.ruby_version = Some(version.into());
        self
    }

    pub fn prefix<T: Into<PathBuf>>(&mut self, prefix: T) -> &mut Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn build_dir<T: Into<PathBuf>>(&mut self, build_dir: T) -> &mut Self {
        self.build_dir = Some(build_dir.into());
        self
    }

    pub fn build(&self) -> Result<(), Box<dyn Error>> {
        self.download_ruby()?;
        self.configure()?;
        self.make()?;
        Ok(())
    }

    fn download_ruby(&self) -> Result<(), Box<dyn Error>> {
        let (version, minor) = if let Some(version) = &self.ruby_version {
            let mut parts = version.splitn(3, '.').take(2);
            let major = parts.next().ok_or("Ruby version not set")?;
            let minor = parts.next().ok_or("Ruby version not set")?;

            (version, format!("{}.{}", major, minor))
        } else {
            return Err("Ruby version not set".into());
        };

        let url = format!(
            "https://cache.ruby-lang.org/pub/ruby/{}/ruby-{}.tar.gz",
            minor, version
        );

        self.sh("curl", &["-L", &url, "-o", "ruby.tar.gz"])?;

        if let Some(sha256) = self.sha256 {
            let output = self.sh("sha256sum", &["-c", "ruby.tar.gz"])?;

            let stdout = String::from_utf8(output.stdout)?;

            if !stdout.contains(sha256) {
                return Err("SHA256 mismatch".into());
            }
        }

        self.sh("tar", &["-xzf", "ruby.tar.gz", "--strip-components=1"])?;

        Ok(())
    }

    fn configure(&self) -> Result<(), Box<dyn Error>> {
        let args: &[&str; 0] = &[];
        self.sh("autoreconf", args)?;
        self.sh(
            "./configure",
            &[
                "--prefix",
                self.prefix.as_ref().unwrap().to_str().unwrap(),
                "--disable-install-doc",
                "--with-ext=",
                "--enable-static",
            ],
        )?;

        Ok(())
    }

    fn make(&self) -> Result<(), Box<dyn Error>> {
        self.sh("make", &["verify-static-library"])?;
        Ok(())
    }

    /// Runs a shell command.
    fn sh<A: AsRef<str>>(&self, cmd: &str, args: &[A]) -> Result<Output, std::io::Error> {
        let mut cmd = Command::new(cmd);
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        if let Some(build_dir) = &self.build_dir {
            cmd.current_dir(build_dir);
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Build directory not set",
            ));
        }

        for arg in args {
            cmd.arg(arg.as_ref());
        }

        cmd.output()
    }
}
