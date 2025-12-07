use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Inlined version of `rb_sys_env::activate` to avoid a circular dependency
    if let Ok(raw_args) = std::env::var("DEP_RB_ENCODED_CARGO_ARGS") {
        let lines = raw_args.split('\x1E');
        let unescaped = lines.map(|line| line.replace('\x1F', "\n"));

        for line in unescaped {
            println!("{line}");
        }
    }

    Ok(())
}
