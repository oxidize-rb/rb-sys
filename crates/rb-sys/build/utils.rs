pub fn is_msvc() -> bool {
    std::env::var("TARGET").unwrap().contains("msvc")
}
