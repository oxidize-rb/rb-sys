use crate::{rb_config, utils::is_msvc};
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    env,
    ffi::OsString,
    fs,
    hash::Hasher,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default, Debug)]
pub struct Build {
    files: Vec<PathBuf>,
    flags: Vec<String>,
}

impl Build {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file(&mut self, file: PathBuf) {
        println!("cargo:rerun-if-changed={}", file.display());
        self.files.push(file);
    }

    pub fn try_compile(self, name: &str) -> Result<()> {
        let (compiler, cc_args) = get_compiler();
        let (archiver, ar_args) = get_archiver();
        let out_dir = PathBuf::from(env::var("OUT_DIR")?).join("cc");
        fs::create_dir_all(&out_dir)?;
        let rb = rb_config();

        let object_files = self.compile_each_file(&compiler, &cc_args, &rb, &out_dir)?;
        let (lib_path, lib_name) =
            self.archive_object_files(&archiver, &ar_args, name, &out_dir, object_files)?;
        self.strip_archived_objects(&archiver, &lib_path)?;

        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static={}", lib_name);

        Ok(())
    }

    fn compile_each_file(
        &self,
        compiler: &str,
        cc_args: &[String],
        rb: &rb_config::RbConfig,
        out_dir: &Path,
    ) -> Result<HashSet<PathBuf>> {
        self.files
            .iter()
            .map(|f| self.compile_file(f, compiler, cc_args, rb, out_dir))
            .collect()
    }

    fn compile_file(
        &self,
        f: &Path,
        compiler: &str,
        cc_args: &[String],
        rb: &rb_config::RbConfig,
        out_dir: &Path,
    ) -> Result<PathBuf> {
        let mut hasher = DefaultHasher::new();
        hasher.write(fs::read(f)?.as_slice());

        let object_file = out_dir
            .join(hasher.finish().to_string())
            .with_extension("o");

        let mut cmd = new_command(compiler);
        cmd.args(cc_args)
            .args(&get_include_args(rb))
            .arg("-c")
            .arg(f)
            .args(&rb.cflags)
            .args(get_common_args())
            .args(&self.flags)
            .args(get_output_file_flag(&object_file));

        run_command(cmd)?;

        Ok(object_file)
    }

    fn archive_object_files(
        &self,
        archiver: &str,
        ar_args: &[String],
        name: &str,
        out_dir: &Path,
        object_files: HashSet<PathBuf>,
    ) -> Result<(PathBuf, String)> {
        let mut cmd = new_command(archiver);
        let mut hasher = DefaultHasher::new();
        object_files
            .iter()
            .for_each(|f| hasher.write(f.to_str().expect("non-utf8 filename").as_bytes()));
        let lib_name = format!("{}-{}", name, hasher.finish());
        let lib_filename = format!("lib{}.a", lib_name);
        let dst = out_dir.join(lib_filename);

        cmd.args(ar_args);

        // The argument structure differs for MSVC and GCC.
        if is_msvc() {
            cmd.arg(format!("/OUT:{}", dst.display()));
            cmd.args(&object_files);
        } else {
            cmd.env("ZERO_AR_DATE", "1").arg("crs").arg(&dst);
            cmd.args(&object_files);
        }

        run_command(cmd)?;

        // The Rust compiler will look for libfoo.a and foo.lib, but the
        // MSVC linker will also be passed foo.lib, so be sure that both
        // exist for now.
        if is_msvc() {
            let lib_dst = dst.with_file_name(format!("{}.lib", lib_name));
            let _ = fs::remove_file(&lib_dst);
            match fs::hard_link(&dst, &lib_dst).or_else(|_| {
                // if hard-link fails, just copy (ignoring the number of bytes written)
                fs::copy(&dst, &lib_dst).map(|_| ())
            }) {
                Ok(_) => (),
                Err(_) => {
                    return Err(
                        "Could not copy or create a hard-link to the generated lib file.".into(),
                    );
                }
            };
        }

        Ok((dst, lib_name))
    }

    fn strip_archived_objects(&self, archiver: &str, libpath: &Path) -> Result<()> {
        let mut cmd = new_command(archiver);

        if is_msvc() {
            cmd.arg("/LTCG").arg(libpath);
        } else {
            cmd.arg("s").arg(libpath);
        }

        run_command(cmd)?;

        Ok(())
    }
}

fn get_include_args(rb: &rb_config::RbConfig) -> Vec<String> {
    vec![
        format!("-I{}", rb.get("rubyhdrdir")),
        format!("-I{}", rb.get("rubyarchhdrdir")),
        format!("-I{}/include/internal", rb.get("rubyhdrdir")),
        format!("-I{}/include/impl", rb.get("rubyhdrdir")),
    ]
}

fn get_common_args() -> Vec<String> {
    fn add_debug_flags(flags: &mut Vec<String>) {
        match env::var("DEBUG") {
            Ok(val) if val == "true" => {
                if is_msvc() {
                    flags.push("-Z7".into());
                } else if cfg!(target_os = "linux") {
                    flags.push("-gdwarf-4".into());
                } else {
                    flags.push("-gdwarf-2".into());
                }
            }
            _ => {}
        }
    }

    fn add_opt_level(flags: &mut Vec<String>) {
        if let Ok(val) = env::var("OPT_LEVEL") {
            match val.as_str() {
                // Msvc uses /O1 to enable all optimizations that minimize code size.
                "z" | "s" | "1" if is_msvc() => flags.push("-O1".into()),
                // -O3 is a valid value for gcc and clang compilers, but not msvc. Cap to /O2.
                "2" | "3" if is_msvc() => flags.push("-O2".into()),
                lvl => flags.push(format!("-O{}", lvl)),
            }
        }
    }

    fn add_compiler_flags(flags: &mut Vec<String>) {
        if !is_msvc() {
            flags.push("-ffunction-sections".into());
            flags.push("-fdata-sections".into());
            flags.push("-fPIC".into());
            flags.push("-fno-omit-frame-pointer".into());
        }
    }

    let mut items = vec![];

    add_debug_flags(&mut items);
    add_compiler_flags(&mut items);
    add_opt_level(&mut items);

    items
}

fn get_compiler() -> (String, Vec<String>) {
    let (name, args) = get_tool("CC", "cc");
    (name, split_arguments(args))
}

fn get_archiver() -> (String, Vec<String>) {
    let (name, args) = get_tool("AR", "ar");
    if name == "libtool" {
        ("ar".into(), vec![])
    } else {
        (name, split_arguments(args))
    }
}

fn get_tool(env_var: &str, default: &str) -> (String, String) {
    let rb = rb_config();
    let tool_args = rb.get(env_var);
    let mut tool_args = tool_args.split_whitespace();
    let tool = tool_args.next().unwrap_or(default);
    let remaining_args = tool_args.collect::<Vec<_>>().join(" ");

    (tool.to_owned(), remaining_args)
}

fn split_arguments(args: String) -> Vec<String> {
    args.split_whitespace().map(Into::into).collect()
}

fn run_command(mut cmd: Command) -> Result<ExitStatus> {
    eprintln!("Running {:?}", cmd);
    let status = cmd.status()?;

    if !status.success() {
        Err(format!("Command '{:?}' failed with status: {}", cmd, status).into())
    } else {
        Ok(status)
    }
}

fn new_command(name: &str) -> Command {
    let mut cmd = Command::new(name);
    cmd.stderr(Stdio::inherit()).stdout(Stdio::inherit());
    cmd
}

fn get_output_file_flag(file: &Path) -> Vec<OsString> {
    if is_msvc() {
        vec![format!("-Fo{}", file.display()).into()]
    } else {
        vec!["-o".into(), file.into()]
    }
}
