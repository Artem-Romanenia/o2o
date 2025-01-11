use test_case::test_case;

#[derive(Debug, PartialEq, Clone)]
pub enum Enum {
    Opt1,
    Opt2,
    Opt3
}

#[derive(Debug, PartialEq, o2o::o2o)]
#[try_map_owned(Enum, String| _ => todo!("unknown"))]
enum EnumDto {
    Opt1,
    Opt2,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Enum2 {
    Opt1,
    Opt2,
    Opt3
}

#[derive(Debug, PartialEq, o2o::o2o)]
#[try_from_owned(Enum2, String| _ => EnumDto2::Opt1)]
enum EnumDto2 {
    Opt1,
    Opt2,
}

#[test_case(Enum::Opt1, EnumDto::Opt1)]
#[test_case(Enum::Opt2, EnumDto::Opt2)]
fn legit_options(l: Enum, r: EnumDto) {
    let e: EnumDto = l.clone().try_into().unwrap();
    assert_eq!(r, e);

    let e: Enum = r.try_into().unwrap();
    assert_eq!(l, e);
}

#[test]
#[should_panic(expected = "unknown")]
fn default_case() {
    let _: EnumDto = Enum::Opt3.try_into().unwrap();
}

#[test_case(Enum2::Opt1, EnumDto2::Opt1)]
#[test_case(Enum2::Opt2, EnumDto2::Opt2)]
#[test_case(Enum2::Opt3, EnumDto2::Opt1)]
fn legit_options_2(l: Enum2, r: EnumDto2) {
    let e: EnumDto2 = l.clone().try_into().unwrap();
    assert_eq!(r, e);
}