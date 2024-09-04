#[test]
fn reqular_repeat() {
    #[derive(Clone, Debug, PartialEq)]
    enum Enum {
        Var1 { field: i32, field_2: i32 },
        Var2 { field_3: i32 },
        Var3 { field_4: i32 },
        Var4 { field_5: i32 },
        Var5 { str: &'static str },
    }

    #[derive(Clone, Debug, PartialEq, o2o::o2o)]
    #[try_map_owned(Enum, String)]
    enum EnumDto {
        Var1 {
            #[o2o(repeat)]
            #[from(~ * 2)]
            #[into(~ / 2)]
            field: i32,
            field_2: i32,
        },
        Var2 {
            field_3: i32,
        },
        Var3 {
            field_4: i32,
        },
        Var4 {
            field_5: i32,
        },
        Var5 {
            #[o2o(stop_repeat)]
            str: &'static str,
        },
    }

    let data = vec![
        (Enum::Var1 { field: 111, field_2: 111 }, EnumDto::Var1 { field: 222, field_2: 222 }),
        (Enum::Var2 { field_3: 222 }, EnumDto::Var2 { field_3: 222 }),
        (Enum::Var3 { field_4: 333 }, EnumDto::Var3 { field_4: 333 }),
        (Enum::Var4 { field_5: 444 }, EnumDto::Var4 { field_5: 444 }),
        (Enum::Var5 { str: "test" }, EnumDto::Var5 { str: "test" }),
    ];

    for data in data {
        let en: Enum = data.1.clone().try_into().unwrap();
        assert_eq!(en, data.0);

        let dto: EnumDto = data.0.try_into().unwrap();
        assert_eq!(dto, data.1);
    }
}

#[test]
fn permeating_repeat() {
    #[derive(Clone, Debug, PartialEq)]
    enum Enum {
        Var1 { field: i32, field_2: i32 },
        Var2 { field_3: i32 },
        Var3 { field_4: i32 },
        Var4 { field_5: i32 },
        Var5 { str: &'static str },
    }

    #[derive(Clone, Debug, PartialEq, o2o::o2o)]
    #[try_map(Enum, String)]
    enum EnumDto {
        Var1 {
            #[o2o(repeat(permeate()))]
            #[from(~ * 2)]
            #[into(~ / 2)]
            field: i32,
            field_2: i32,
        },
        Var2 {
            field_3: i32,
        },
        Var3 {
            field_4: i32,
        },
        Var4 {
            field_5: i32,
        },
        Var5 {
            #[o2o(stop_repeat)]
            str: &'static str,
        },
    }

    let data = vec![
        (Enum::Var1 { field: 111, field_2: 111 }, EnumDto::Var1 { field: 222, field_2: 222 }),
        (Enum::Var2 { field_3: 222 }, EnumDto::Var2 { field_3: 444 }),
        (Enum::Var3 { field_4: 333 }, EnumDto::Var3 { field_4: 666 }),
        (Enum::Var4 { field_5: 444 }, EnumDto::Var4 { field_5: 888 }),
        (Enum::Var5 { str: "test" }, EnumDto::Var5 { str: "test" }),
    ];

    for data in data {
        let dto_ref = &data.1;
        let en: Enum = dto_ref.try_into().unwrap();
        assert_eq!(en, data.0);

        let en: Enum = data.1.clone().try_into().unwrap();
        assert_eq!(en, data.0);

        let en_ref = &data.0;
        let dto: EnumDto = en_ref.try_into().unwrap();
        assert_eq!(dto, data.1);

        let dto: EnumDto = data.0.try_into().unwrap();
        assert_eq!(dto, data.1);
    }
}
