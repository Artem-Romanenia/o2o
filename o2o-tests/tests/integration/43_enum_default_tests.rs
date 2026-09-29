#[derive(Clone, PartialEq)]
enum Enum {
    Var1,
    Var22,
}

#[derive(Clone, PartialEq, o2o::o2o)]
#[map(Enum| _ => panic!("unknown"))]
enum EnumDto {
    Var1,
    #[map(Var22)]
    Var2,
    #[ghost]
    Var3,
    #[ghost]
    Var4 {
        _str: String,
        _i: i32,
    },
    #[ghost]
    Var5(i32, String),
}

#[derive(Clone, PartialEq, o2o::o2o)]
#[map(EnumDto2| _ => panic!("unknown"))]
#[ghosts]
enum Enum2 {
    Var1,
    #[map(Var2)]
    Var22,
}

#[derive(Clone, PartialEq)]
enum EnumDto2 {
    Var1,
    Var2,
    Var3,
    Var4 { _str: String, _i: i32 },
    Var5(i32, String),
}

#[test]
fn enum2enum() {
    for data in vec![(Enum::Var1, EnumDto::Var1), (Enum::Var22, EnumDto::Var2)] {
        let dto_ref = &data.1;
        let en: Enum = dto_ref.into();
        assert!(en == data.0);

        let en: Enum = data.1.clone().into();
        assert!(en == data.0);

        let en_ref = &data.0;
        let dto: EnumDto = en_ref.into();
        assert!(dto == data.1);

        let dto: EnumDto = data.0.into();
        assert!(dto == data.1);
    }
}

#[test]
#[should_panic(expected = "unknown")]
fn enum2enum_panic() {
    let dto = EnumDto::Var3;
    let _: Enum = dto.into();
}

#[test]
#[should_panic(expected = "unknown")]
fn enum2enum_panic_2() {
    let dto = EnumDto::Var4 { _str: "test".into(), _i: 123 };
    let _: Enum = dto.into();
}

#[test]
#[should_panic(expected = "unknown")]
fn enum2enum_panic_3() {
    let dto = EnumDto::Var5(123, "test".into());
    let _: Enum = dto.into();
}

#[test]
fn enum2enum_reverse() {
    for data in vec![(Enum2::Var1, EnumDto2::Var1), (Enum2::Var22, EnumDto2::Var2)] {
        let dto_ref = &data.1;
        let en: Enum2 = dto_ref.into();
        assert!(en == data.0);

        let en: Enum2 = data.1.clone().into();
        assert!(en == data.0);

        let en_ref = &data.0;
        let dto: EnumDto2 = en_ref.into();
        assert!(dto == data.1);

        let dto: EnumDto2 = data.0.into();
        assert!(dto == data.1);
    }
}

#[test]
#[should_panic(expected = "unknown")]
fn enum2enum_panic_reverse() {
    let dto = EnumDto2::Var3;
    let _: Enum2 = dto.into();
}

#[test]
#[should_panic(expected = "unknown")]
fn enum2enum_panic_reverse_2() {
    let dto = EnumDto2::Var4 { _str: "test".into(), _i: 123 };
    let _: Enum2 = dto.into();
}

#[test]
#[should_panic(expected = "unknown")]
fn enum2enum_panic_reverse_3() {
    let dto = EnumDto2::Var5(123, "test".into());
    let _: Enum2 = dto.into();
}
