enum Enum {
    Item1,
    Item2,
}

enum EnumDto {
    Item1,
    Item2,
}

impl From<Enum> for EnumDto {
    fn from(value: Enum) -> EnumDto {
        match value {
            Enum::Item1 => EnumDto::Item1,
            Enum::Item2 => EnumDto::Item2,
        }
    }
}

impl Into<Enum> for EnumDto {
    fn into(self) -> Enum {
        match self {
            EnumDto::Item1 => Enum::Item1,
            EnumDto::Item2 => Enum::Item2,
        }
    }
}

impl From<&Enum> for EnumDto {
    fn from(value: &Enum) -> Self {
        match value {
            Enum::Item1 => EnumDto::Item1,
            Enum::Item2 => EnumDto::Item2,
        }
    }
}

impl Into<Enum> for &EnumDto {
    fn into(self) -> Enum {
        match self {
            EnumDto::Item1 => Enum::Item1,
            EnumDto::Item2 => Enum::Item2,
        }
    }
}

enum EnumWithData {
    Item1(String),
    Item2(i32),
}

enum EnumWithDataDto {
    Item1(String),
    Item2(i32),
}

impl From<EnumWithData> for EnumWithDataDto {
    fn from(value: EnumWithData) -> EnumWithDataDto {
        match value {
            EnumWithData::Item1(str) => EnumWithDataDto::Item1(str),
            EnumWithData::Item2(i) => EnumWithDataDto::Item2(i),
        }
    }
}

impl Into<EnumWithData> for EnumWithDataDto {
    fn into(self) -> EnumWithData {
        match self {
            EnumWithDataDto::Item1(str) => EnumWithData::Item1(str),
            EnumWithDataDto::Item2(i) => EnumWithData::Item2(i),
        }
    }
}

impl From<&EnumWithData> for EnumWithDataDto {
    fn from(value: &EnumWithData) -> EnumWithDataDto {
        match value {
            EnumWithData::Item1(str) => EnumWithDataDto::Item1(str.clone()),
            EnumWithData::Item2(i) => EnumWithDataDto::Item2(*i),
        }
    }
}

#[derive(o2o::o2o)]
#[from_owned(Test2)]
enum Test {
    Opt1(i32, String),
    Opt2 { val: i32, str: String}
}

enum Test2 {
    Opt1(i32, String),
    Opt2 { val: i32, str: String}
}

struct Entity {
    item_1: String,
    item_2: i32,
}

#[derive(o2o::o2o)]
#[map_owned(Entity)]
struct EntityDto {
    #[map(~.clone())]
    item_1: String,
    item_2: i32,
}