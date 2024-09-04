#[derive(PartialEq, Eq)]
enum Enum {
    Item1,
    Item2,
}

#[derive(Clone, PartialEq, Eq, o2o::o2o)]
#[try_map(Enum, String)]
enum EnumDto {
    Item1,
    #[map(Item2)]
    Item2Dto,
}

#[derive(PartialEq, Eq)]
enum EnumWithData {
    Item1(i32, i16),
    Item2 { str: String, i: i32 },
}

#[derive(Clone, PartialEq, Eq, o2o::o2o)]
#[try_map_owned(EnumWithData, String)]
enum EnumWithDataDto {
    #[map(Item1)]
    Item1Dto(i32, i16),
    Item2 {
        #[map(str)]
        string: String,
        i: i32,
    },
}

#[test]
fn enum2enum() {
    for data in vec![(EnumDto::Item1, Enum::Item1), (EnumDto::Item2Dto, Enum::Item2)] {
        let dto_ref = &data.0;
        let en: Enum = dto_ref.try_into().unwrap();
        assert!(en == data.1);

        let en: Enum = data.0.clone().try_into().unwrap();
        assert!(en == data.1);

        let en_ref = &data.1;
        let dto: EnumDto = en_ref.try_into().unwrap();
        assert!(dto == data.0);

        let dto: EnumDto = data.1.try_into().unwrap();
        assert!(dto == data.0);
    }
}

#[test]
fn enum2enum_with_data() {
    for data in vec![
        (EnumWithDataDto::Item1Dto(123, 321), EnumWithData::Item1(123, 321)),
        (EnumWithDataDto::Item2 { string: "Test".into(), i: 654 }, EnumWithData::Item2 { str: "Test".into(), i: 654 }),
    ] {
        let en: EnumWithData = data.0.clone().try_into().unwrap();
        assert!(en == data.1);

        let dto: EnumWithDataDto = data.1.try_into().unwrap();
        assert!(dto == data.0);
    }
}
