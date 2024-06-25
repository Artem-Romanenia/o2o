#[derive(Clone, PartialEq, Eq, o2o::o2o)]
#[map(EnumDto)]
enum Enum {
    Var1,
    #[type_hint(as ())]
    Var2 {
        #[map_ref(*~)]field: i32,
        #[map_ref(~.clone())]str_field: String
    },
    #[type_hint(as {})]
    Var3(
        #[map_owned(field)]
        #[map_ref(field, *~)]
        i32,
        
        #[map_owned(str_field)]
        #[map_ref(str_field, ~.clone())]
        String
    )
}

#[derive(Clone, PartialEq, Eq)]
enum EnumDto {
    Var1,
    Var2(i32, String),
    Var3 {field: i32, str_field: String}
}

#[test]
fn enum2enum() {
    for data in vec![
        (EnumDto::Var1, Enum::Var1),
        (EnumDto::Var2(123, "test".into()), Enum::Var2 { field: 123, str_field: "test".into() }),
        (EnumDto::Var3 { field: 123, str_field: "test".into() }, Enum::Var3(123, "test".into()))
    ] {
        let dto_ref = &data.0;
        let en: Enum = dto_ref.into();
        assert!(en == data.1);

        let en: Enum = data.0.clone().into();
        assert!(en == data.1);

        let en_ref = &data.1;
        let dto: EnumDto = en_ref.into();
        assert!(dto == data.0);

        let dto: EnumDto = data.1.into();
        assert!(dto == data.0);
    }
}