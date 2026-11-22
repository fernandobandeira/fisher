use uci::{Input, parse_command};

#[test]
fn ignored_commands_consume_the_rest_of_the_line() {
    for line in [
        "setoption name Path value quit",
        "debug on quit",
        "ponderhit quit",
        "register name quit",
    ] {
        assert_eq!(parse_command(line), Input::Ignored, "{line}");
    }
}
