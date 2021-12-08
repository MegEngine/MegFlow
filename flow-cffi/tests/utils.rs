use std::env;
use std::path::PathBuf;

lazy_static::lazy_static!(
    static ref LIB_PATH: PathBuf = test_cdylib::build_current_project();
);

pub fn ready_cpp() {
    let mut crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    crate_dir.push("ffi");
    let mut lib_dir = LIB_PATH.clone();
    lib_dir.pop();
    env::set_var(
        "INLINE_C_RS_CFLAGS",
        format!(
            "-I {} -L {}",
            crate_dir.to_str().unwrap(),
            lib_dir.to_str().unwrap()
        )
        .as_str(),
    );
    env::set_var("INLINE_C_RS_LDFLAGS", LIB_PATH.to_str().unwrap());
}
