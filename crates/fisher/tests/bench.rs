use std::{error::Error, process::Command};

#[test]
fn bench_argument_reports_the_published_signature_and_exits() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_fisher"))
        .arg("bench")
        .stdin(std::process::Stdio::null())
        .output()?;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stderr, [], "{output:?}");
    // Sum of the published counts at plies 1 through 4 in expectations/perft.epd.
    assert_eq!(
        String::from_utf8(output.stdout)?.lines().last(),
        Some("Nodes searched  : 11024485"),
    );
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
