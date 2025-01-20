use test_case::test_case;

#[derive(Debug, PartialEq, o2o::o2o)]
#[from_owned(String| match @.as_str(), _ => todo!())]
#[from_owned(i32| repeat(), match @.to_string().as_str(), _ => todo!())]
#[from_owned(f32)]
pub enum Enum {
    #[literal("1")]
    Opt1,
    #[literal("2")]
    Opt2,
}

#[test_case(Enum::Opt1, "1".into())]
#[test_case(Enum::Opt2, "2".into())]
fn legit_options(l: Enum, r: String) {
    let e: Enum = r.into();
    assert_eq!(l, e);
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn default_case() {
    let str: String = "3".into();
    let _: Enum = str.into();
}

#[test_case(Enum::Opt1, 1)]
#[test_case(Enum::Opt2, 2)]
fn legit_options_i32(l: Enum, r: i32) {
    let e: Enum = r.into();
    assert_eq!(l, e);
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn default_case_i32() {
    let _: Enum = 3.into();
}

#[test_case(Enum::Opt1, 1.0)]
#[test_case(Enum::Opt2, 2.0)]
fn legit_options_f32(l: Enum, r: f32) {
    let e: Enum = r.into();
    assert_eq!(l, e);
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn default_case_f32() {
    let _: Enum = 3.into();
}