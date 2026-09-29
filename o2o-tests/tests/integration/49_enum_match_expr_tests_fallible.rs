use test_case::test_case;

#[derive(Debug, PartialEq, o2o::o2o)]
#[try_from_owned(String, String| match @.as_str(), _ => Err("error")?)]
#[try_from_owned(i32, String| repeat(), match @.to_string().as_str(), _ => Err("error")?)]
#[try_from_owned(f32, String)]
pub enum Enum {
    #[literal("1")]
    Opt1,
    #[literal("2")]
    Opt2,
}

#[test_case(Ok(Enum::Opt1), "1".into())]
#[test_case(Ok(Enum::Opt2), "2".into())]
#[test_case(Err("error".into()), "3".into())]
fn legit_options(l: Result<Enum, String>, r: String) {
    let e: Result<Enum, String> = r.try_into();
    assert_eq!(l, e);
}

#[test_case(Ok(Enum::Opt1), 1)]
#[test_case(Ok(Enum::Opt2), 2)]
#[test_case(Err("error".into()), 3)]
fn legit_options_i32(l: Result<Enum, String>, r: i32) {
    let e: Result<Enum, String> = r.try_into();
    assert_eq!(l, e);
}

#[test_case(Ok(Enum::Opt1), 1.0)]
#[test_case(Ok(Enum::Opt2), 2.0)]
#[test_case(Err("error".into()), 3.0)]
fn legit_options_f32(l: Result<Enum, String>, r: f32) {
    let e: Result<Enum, String> = r.try_into();
    assert_eq!(l, e);
}