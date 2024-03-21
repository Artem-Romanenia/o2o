#[derive(PartialEq, Eq)]
enum EnumWithData {
    Item1(i32, i16),
    Item2 { str: String, i: i32 },
}

#[derive(Clone, PartialEq, Eq, o2o::o2o)]
#[map_owned(EnumWithData)]
enum EnumWithDataDto {
    Item1(#[map(1)]i16, #[map(0)]i32),
    Item2 {
        #[map(str)]
        string: String, 
        i: i32 
    },
}

#[test]
fn named2named_with_data() {
    for data in vec![
        (EnumWithDataDto::Item1(123, 321), EnumWithData::Item1(321, 123)),
        (EnumWithDataDto::Item2 { string: "Test".into(), i: 654 }, EnumWithData::Item2 { str: "Test".into(), i: 654 })
    ] {
        let en: EnumWithData = data.0.clone().into();
        assert!(en == data.1);

        let dto: EnumWithDataDto = data.1.into();
        assert!(dto == data.0);
    }
}