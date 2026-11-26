use std::{ffi::OsStr, io, process::Command};

#[test]
fn bench_argument_reports_the_library_signature_and_exits() -> io::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_fisher"))
        .arg("bench")
        .stdin(std::process::Stdio::null())
        .output()?;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stderr, [], "{output:?}");
    let mut expected = Vec::new();
    assert!(bench::run_if_requested(
        Some(OsStr::new("bench")),
        &mut expected
    )?);
    assert_eq!(output.stdout, expected);
    Ok(())
}

#[test]
fn bench_write_error_exits_with_failure() -> std::io::Result<()> {
    let (reader, writer) = std::io::pipe()?;
    drop(reader);
    let status = Command::new(env!("CARGO_BIN_EXE_fisher"))
        .arg("bench")
        .stdout(writer)
        .stderr(std::process::Stdio::null())
        .status()?;
    assert_eq!(status.code(), Some(1));
    Ok(())
}
