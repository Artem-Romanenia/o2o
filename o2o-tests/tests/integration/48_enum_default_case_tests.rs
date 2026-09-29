use test_case::test_case;

#[derive(Debug, PartialEq, Clone)]
pub enum Enum {
    Opt1,
    Opt2,
    Opt3
}

#[derive(Debug, PartialEq, o2o::o2o)]
#[map_owned(Enum| _ => todo!("unknown"))]
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
#[from_owned(Enum2| _ => EnumDto2::Opt1)]
enum EnumDto2 {
    Opt1,
    Opt2,
}

#[test_case(Enum::Opt1, EnumDto::Opt1)]
#[test_case(Enum::Opt2, EnumDto::Opt2)]
fn legit_options(l: Enum, r: EnumDto) {
    let e: EnumDto = l.clone().into();
    assert_eq!(r, e);

    let e: Enum = r.into();
    assert_eq!(l, e);
}

#[test]
#[should_panic(expected = "unknown")]
fn default_case() {
    let _: EnumDto = Enum::Opt3.into();
}

#[test_case(Enum2::Opt1, EnumDto2::Opt1)]
#[test_case(Enum2::Opt2, EnumDto2::Opt2)]
#[test_case(Enum2::Opt3, EnumDto2::Opt1)]
fn legit_options_2(l: Enum2, r: EnumDto2) {
    let e: EnumDto2 = l.clone().into();
    assert_eq!(r, e);
}