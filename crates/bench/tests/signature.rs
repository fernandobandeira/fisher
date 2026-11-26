use std::{error::Error, ffi::OsStr};

#[test]
fn reports_the_depth_four_signature_within_the_published_bounds() -> Result<(), Box<dyn Error>> {
    let mut output = Vec::new();
    assert!(bench::run_if_requested(
        Some(OsStr::new("bench")),
        &mut output
    )?);

    let output = String::from_utf8(output)?;
    let mut lines = output.lines();
    // Published perft sums through depth 4 and the ceiling set in D39.
    let exhaustive = [206_603, 4_185_552, 46_255, 432_070, 2_167_396, 3_986_609];
    let mut total = 0;
    for (index, upper_bound) in exhaustive.into_iter().enumerate() {
        let line = lines.next().ok_or("missing position line")?;
        let nodes = assert_position_signature(index + 1, line, upper_bound)?;
        total += nodes;
    }
    assert!(total <= 1_102_448, "{total}");
    assert_eq!(
        lines.next(),
        Some(format!("Nodes searched  : {total}").as_str())
    );
    assert_eq!(lines.next(), None);

    // This revision's measured signature is the "previous revision" baseline
    // in docs/evidence.md for later changes. The outside values are the bounds above.
    assert_eq!(
        output,
        concat!(
            "Position 1: 2315\n",
            "Position 2: 10714\n",
            "Position 3: 1137\n",
            "Position 4: 6165\n",
            "Position 5: 22595\n",
            "Position 6: 51252\n",
            "Nodes searched  : 94178\n",
        )
    );
    Ok(())
}

fn assert_position_signature(
    number: usize,
    line: &str,
    upper_bound: u64,
) -> Result<u64, Box<dyn Error>> {
    let prefix = format!("Position {number}: ");
    let nodes = line
        .strip_prefix(&prefix)
        .ok_or("wrong position")?
        .parse()?;
    assert!((1..=upper_bound).contains(&nodes), "{line}");

    Ok(nodes)
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
