#[derive(Clone, PartialEq)]
enum Enum {
    Var1,
    Var22,
    Error(String),
}

#[derive(Clone, PartialEq, o2o::o2o)]
#[map(Enum)]
enum EnumDto {
    Var1,
    #[map(Var22)]
    Var2,
    #[type_hint(as())]
    #[into(Error, ~(_str.clone()))]
    #[from(Error, ~{ _str: f0.clone(), _i: 123})]
    Var3 {
        _str: String,
        #[ghost]
        _i: i32,
    },
}

#[derive(Clone, PartialEq, o2o::o2o)]
#[map(EnumDto2)]
enum Enum2 {
    Var1,
    #[map(Var2)]
    Var22,
    #[type_hint(as {})]
    #[into(Var3, ~{ _str: f0.clone(), _i: 123})]
    #[from(Var3, ~(_str.clone()))]
    #[ghosts(_i: {})]
    Error(#[map(_str)] String),
}

#[derive(Clone, PartialEq)]
enum EnumDto2 {
    Var1,
    Var2,
    Var3 { _str: String, _i: i32 },
}

#[test]
fn enum2enum() {
    for data in vec![
        (Enum::Var1, EnumDto::Var1),
        (Enum::Var22, EnumDto::Var2),
        (
            Enum::Error("test".into()),
            EnumDto::Var3 {
                _str: "test".into(),
                _i: 123,
            },
        ),
    ] {
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
fn enum2enum_reverse() {
    for data in vec![
        (Enum2::Var1, EnumDto2::Var1),
        (Enum2::Var22, EnumDto2::Var2),
        (
            Enum2::Error("test".into()),
            EnumDto2::Var3 {
                _str: "test".into(),
                _i: 123,
            },
        ),
    ] {
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
