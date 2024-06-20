#[derive(PartialEq, Eq)]
enum EnumWithData {
    Item1(i32, i16),
    Item2 { str: String, i: i32 },
}

#[derive(Clone, PartialEq, Eq, o2o::o2o)]
#[try_map_owned(EnumWithData, String)]
enum EnumWithDataDto {
    Item1(
        i32, 
        #[from_owned(~ * 2)]
        #[owned_into(~ / 2)]
        i16
    ),
    Item2 {
        str: String,
        #[from(~.to_string())] 
        #[into(~.parse::<i32>().unwrap())]
        i: String 
    },
}

#[test]
fn enum2enum_with_data() {
    for data in vec![
        (EnumWithDataDto::Item1(123, 222), EnumWithData::Item1(123, 111)),
        (EnumWithDataDto::Item2 { str: "Test".into(), i: "654".into() }, EnumWithData::Item2 { str: "Test".into(), i: 654 })
    ] {
        let en: EnumWithData = data.0.clone().try_into().unwrap();
        assert!(en == data.1);

        let dto: EnumWithDataDto = data.1.try_into().unwrap();
        assert!(dto == data.0);
    }
}