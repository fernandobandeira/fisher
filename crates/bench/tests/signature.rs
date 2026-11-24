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
