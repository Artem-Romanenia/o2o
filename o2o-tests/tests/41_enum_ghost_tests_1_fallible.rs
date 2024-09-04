#[derive(Clone, PartialEq)]
enum Enum {
    Var1,
    Var22,
}

#[derive(Clone, PartialEq, o2o::o2o)]
#[try_map(Enum, String)]
enum EnumDto {
    Var1,
    #[map(Var22)]
    Var2,
    #[ghost({Err("impl var3")?})]
    Var3,
    #[ghost({Err("impl var4")?})]
    Var4 {
        _str: String,
        _i: i32,
    },
    #[ghost({Err("impl var5")?})]
    Var5(i32, String),
}

#[derive(Clone, PartialEq, o2o::o2o)]
#[try_map(EnumDto2, String)]
#[ghosts(Var3: {Err("impl var3")?}, Var4 { .. }: {Err("impl var4")?}, Var5(..): {Err("impl var5")?})]
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
        let en: Enum = dto_ref.try_into().unwrap();
        assert!(en == data.0);

        let en: Enum = data.1.clone().try_into().unwrap();
        assert!(en == data.0);

        let en_ref = &data.0;
        let dto: EnumDto = en_ref.try_into().unwrap();
        assert!(dto == data.1);

        let dto: EnumDto = data.0.try_into().unwrap();
        assert!(dto == data.1);
    }
}

#[test]
fn enum2enum_panic() {
    let dto = EnumDto::Var3;
    let res: Result<Enum, String> = dto.try_into();
    assert!(res.is_err_and(|x| x == "impl var3"))
}

#[test]
fn enum2enum_panic_2() {
    let dto = EnumDto::Var4 { _str: "test".into(), _i: 123 };
    let res: Result<Enum, String> = dto.try_into();
    assert!(res.is_err_and(|x| x == "impl var4"))
}

#[test]
fn enum2enum_panic_3() {
    let dto = EnumDto::Var5(123, "test".into());
    let res: Result<Enum, String> = dto.try_into();
    assert!(res.is_err_and(|x| x == "impl var5"))
}

#[test]
fn enum2enum_reverse() {
    for data in vec![(Enum2::Var1, EnumDto2::Var1), (Enum2::Var22, EnumDto2::Var2)] {
        let dto_ref = &data.1;
        let en: Enum2 = dto_ref.try_into().unwrap();
        assert!(en == data.0);

        let en: Enum2 = data.1.clone().try_into().unwrap();
        assert!(en == data.0);

        let en_ref = &data.0;
        let dto: EnumDto2 = en_ref.try_into().unwrap();
        assert!(dto == data.1);

        let dto: EnumDto2 = data.0.try_into().unwrap();
        assert!(dto == data.1);
    }
}

#[test]
fn enum2enum_panic_reverse() {
    let dto = EnumDto2::Var3;
    let res: Result<Enum2, String> = dto.try_into();
    assert!(res.is_err_and(|x| x == "impl var3"))
}

#[test]
fn enum2enum_panic_reverse_2() {
    let dto = EnumDto2::Var4 { _str: "test".into(), _i: 123 };
    let res: Result<Enum2, String> = dto.try_into();
    assert!(res.is_err_and(|x| x == "impl var4"))
}

#[test]
fn enum2enum_panic_reverse_3() {
    let dto = EnumDto2::Var5(123, "test".into());
    let res: Result<Enum2, String> = dto.try_into();
    assert!(res.is_err_and(|x| x == "impl var5"))
}
