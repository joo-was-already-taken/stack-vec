use std::process::Command;

#[test]
/// Tests if library can be compiled with `package.rust-version` in *Cargo.toml*
/// with `cargo msrv verify`.
fn msrv() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .arg("msrv")
        .arg("verify")
        .output()?;

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr)?;
        println!("{}", stderr);
        let stdout = std::str::from_utf8(&output.stdout)?;
        println!("{}", stdout);
        panic!();
    }

    Ok(())
}
