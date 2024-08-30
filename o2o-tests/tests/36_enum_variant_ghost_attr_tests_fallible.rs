#[derive(Clone, PartialEq, o2o::o2o)]
#[try_map(EnumDto, String)]
enum Enum {
    Var1,
    Var2 {
        #[map_ref(*~)]
        field: i32,
        #[ghost(321.0)]
        _f: f32,
        #[map_ref(~.clone())]
        str_field: String,
    },
    Var3(
        #[map_ref(*~)] i32,
        #[ghost({123.0})] f32,
        #[map_ref(~.clone())] String,
    ),
}

#[derive(Clone, PartialEq)]
enum EnumDto {
    Var1,
    Var2 { field: i32, str_field: String },
    Var3(i32, String),
}

#[test]
fn enum2enum() {
    for data in vec![
        (Enum::Var1, EnumDto::Var1),
        (
            Enum::Var2 {
                field: 123,
                _f: 321.0,
                str_field: "test".into(),
            },
            EnumDto::Var2 {
                field: 123,
                str_field: "test".into(),
            },
        ),
        (
            Enum::Var3(123, 123.0, "test".into()),
            EnumDto::Var3(123, "test".into()),
        ),
    ] {
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
