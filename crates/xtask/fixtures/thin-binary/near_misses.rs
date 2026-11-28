#[doc = "Fixture wiring."]

fn main() {
    // fn pretend() {}
    let wiring: fn() = || {};
    wiring();
    let _description = "fn pretend() {}";
    let include = r#"mod hidden; impl Parser {} include!("other.rs");"#;
    let r#mod = include;
    let _ = r#mod;
}
