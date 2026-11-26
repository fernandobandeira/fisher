use std::{error::Error, ffi::OsStr};

#[test]
fn reports_the_depth_four_signature_from_published_perft_counts() -> Result<(), Box<dyn Error>> {
    let mut output = Vec::new();
    assert!(bench::run_if_requested(
        Some(OsStr::new("bench")),
        &mut output
    )?);

    Ok(())
}

#[test]
fn ignores_unrelated_arguments_without_writing() -> std::io::Result<()> {
    let mut output = Vec::new();
    for argument in [None, Some(OsStr::new("uci"))] {
        assert!(!bench::run_if_requested(argument, &mut output)?);
    }
    assert_eq!(output, []);
    Ok(())
}

#[test]
fn reports_a_write_failure() {
    use std::io::{self, Write};

    struct ClosedOutput;
    impl Write for ClosedOutput {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let error = bench::run_if_requested(Some(OsStr::new("bench")), &mut ClosedOutput).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
}
