use std::{error::Error, ffi::OsStr};

#[test]
fn reports_the_depth_four_signature_from_published_perft_counts() -> Result<(), Box<dyn Error>> {
    let mut output = Vec::new();
    assert!(bench::run_if_requested(
        Some(OsStr::new("bench")),
        &mut output
    )?);

    // fisher-ssg sums the published perft counts at plies 1 through 4.
    assert_eq!(
        String::from_utf8(output)?,
        concat!(
            "Position 1: 206603\n",
            "Position 2: 4185552\n",
            "Position 3: 46255\n",
            "Position 4: 432070\n",
            "Position 5: 2167396\n",
            "Position 6: 3986609\n",
            "Nodes searched  : 11024485\n",
        )
    );
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
