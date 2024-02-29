#[derive(PartialEq, Eq, Clone)]
enum Enum {
    Item1,
    Item2,
}

#[derive(PartialEq, Eq, Clone, o2o::o2o)]
#[map(Enum)]
enum EnumDto {
    Item1,
    Item2,
}

// #[derive(PartialEq, Eq)]
// enum EnumWithData {
//     Item1(i32, i16),
//     Item2 { str: String, i: i32 },
// }

// #[derive(PartialEq, Eq, o2o::o2o)]
// #[map(EnumWithData)]
// enum EnumWithDataDto {
//     Item1(i32, i16),
//     Item2 { str: String, i: i32 },
// }

#[test]
fn named2named() {
    for data in vec![
        (EnumDto::Item1, Enum::Item1),
        (EnumDto::Item2, Enum::Item2)
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