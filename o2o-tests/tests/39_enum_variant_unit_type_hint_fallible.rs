#[test]
fn struct2unit() {
    #[derive(Clone, PartialEq, Eq, o2o::o2o)]
    #[try_map(EnumDto, String)]
    enum Enum {
        Var1,
        #[type_hint(as Unit)]
        Var2(#[ghost(123)]i32, #[ghost({"test".into()})]String),
        #[type_hint(as Unit)]
        Var3 {#[ghost(123)]_field: i32, #[ghost({"test".into()})]_str_field: String}
    }

    #[derive(Clone, PartialEq, Eq)]
    enum EnumDto {
        Var1,
        Var2,
        Var3
    }

    for data in vec![
        (Enum::Var1, EnumDto::Var1),
        (Enum::Var2(123, "test".into()), EnumDto::Var2),
        (Enum::Var3 { _field: 123, _str_field: "test".into() }, EnumDto::Var3)
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

#[test]
fn unit2struct() {
    #[derive(Clone, PartialEq, Eq)]
    enum Enum {
        Var1,
        Var2(i32, String),
        Var3 { _field: i32, _str_field: String }
    }
    
    #[derive(Clone, PartialEq, Eq, o2o::o2o)]
    #[try_map(Enum, String)]
    enum EnumDto {
        Var1,
        #[type_hint(as ())]
        #[ghosts(0: {123}, 1: {"test".into()})]
        Var2,
        #[type_hint(as {})]
        #[ghosts(_field: {123}, _str_field: {"test".into()})]
        Var3
    }

    for data in vec![
        (Enum::Var1, EnumDto::Var1),
        (Enum::Var2(123, "test".into()), EnumDto::Var2),
        (Enum::Var3 { _field: 123, _str_field: "test".into() }, EnumDto::Var3)
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

#[test]
fn struct2unit_no_ghost() {
    #[derive(Clone, PartialEq, Eq, o2o::o2o)]
    #[try_into(EnumDto, String)]
    enum Enum {
        Var1,
        #[type_hint(as Unit)]
        Var2(i32, String),
        #[type_hint(as Unit)]
        Var3 {_field: i32, _str_field: String}
    }

    #[derive(Clone, PartialEq, Eq)]
    enum EnumDto {
        Var1,
        Var2,
        Var3
    }

    for data in vec![
        (Enum::Var1, EnumDto::Var1),
        (Enum::Var2(123, "test".into()), EnumDto::Var2),
        (Enum::Var3 { _field: 123, _str_field: "test".into() }, EnumDto::Var3)
    ] {
        let en_ref = &data.0;
        let dto: EnumDto = en_ref.try_into().unwrap();
        assert!(dto == data.1);

        let dto: EnumDto = data.0.try_into().unwrap();
        assert!(dto == data.1);
    }
}

#[test]
fn unit2struct_no_ghost() {
    #[derive(Clone, PartialEq, Eq)]
    enum Enum {
        Var1,
        Var2(i32, String),
        Var3 { _field: i32, _str_field: String }
    }
    
    #[derive(Clone, PartialEq, Eq, o2o::o2o)]
    #[try_from(Enum, String)]
    enum EnumDto {
        Var1,
        #[type_hint(as ())] Var2,
        #[type_hint(as {})] Var3
    }

    for data in vec![
        (Enum::Var1, EnumDto::Var1),
        (Enum::Var2(123, "test".into()), EnumDto::Var2),
        (Enum::Var3 { _field: 123, _str_field: "test".into() }, EnumDto::Var3)
    ] {
        let en_ref = &data.0;
        let dto: EnumDto = en_ref.try_into().unwrap();
        assert!(dto == data.1);

        let dto: EnumDto = data.0.try_into().unwrap();
        assert!(dto == data.1);
    }
}