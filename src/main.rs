use std::process::ExitCode;

use std::io;

use rs_tars2zips::stdin2tarnames2zip_default;

fn env_val_by_key(key: &'static str) -> impl FnMut() -> Result<String, io::Error> {
    move || {
        std::env::var(key)
            .map_err(|e| format!("env val {key} missing: {e}"))
            .map_err(io::Error::other)
    }
}

fn env2outzip_dir() -> Result<String, io::Error> {
    env_val_by_key("ENV_OUTPUT_ZIPS_DIR")()
}

fn sub() -> Result<(), io::Error> {
    let ozdir: String = env2outzip_dir()?;
    stdin2tarnames2zip_default(ozdir)
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
