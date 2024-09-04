#[derive(PartialEq, Eq)]
enum EnumWithData {
    Item1(i32, i16),
    Item2 { str: String, i: i32 },
}

#[derive(Clone, PartialEq, Eq, o2o::o2o)]
#[map_owned(EnumWithData)]
enum EnumWithDataDto {
    Item1(
        #[from(0, ~.to_string())]
        #[into(0, ~.parse::<i32>().unwrap())]
        String,
        i16,
    ),
    Item2 {
        str: String,
        #[from(i, ~.to_string())]
        #[into(i, ~.parse::<i32>().unwrap())]
        i_str: String,
    },
}

#[test]
fn enum2enum_with_data() {
    for data in vec![
        (EnumWithDataDto::Item1("123".into(), 321), EnumWithData::Item1(123, 321)),
        (EnumWithDataDto::Item2 { str: "Test".into(), i_str: "654".into() }, EnumWithData::Item2 { str: "Test".into(), i: 654 }),
    ] {
        let en: EnumWithData = data.0.clone().into();
        assert!(en == data.1);

        let dto: EnumWithDataDto = data.1.into();
        assert!(dto == data.0);
    }
}
