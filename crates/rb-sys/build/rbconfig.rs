use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::process::Command;
use std::sync::Mutex;

lazy_static! {
    static ref CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub fn rbconfig(key: &str) -> String {
    let mut cache = CACHE.lock().unwrap();
    let cache_key = String::from(key);

    if cache.get(&cache_key).is_some() {
        return cache.get(&cache_key).unwrap().to_owned();
    }

    println!("cargo:rerun-if-env-changed=RBCONFIG_{}", key);

    match env::var(format!("RBCONFIG_{}", key)) {
        Ok(val) => val,
        Err(_) => {
            let ruby = env::var_os("RUBY").unwrap_or_else(|| OsString::from("ruby"));

            let config = Command::new(ruby)
                .arg("--disable-gems")
                .arg("-rrbconfig")
                .arg("-e")
                .arg(format!("print RbConfig::CONFIG['{}']", key))
                .output()
                .unwrap_or_else(|e| panic!("ruby not found: {}", e));

            let val = String::from_utf8(config.stdout).expect("RbConfig value not UTF-8!");
            cache.insert(cache_key, val.clone());
            val
        }
    }
}
