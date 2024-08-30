#[derive(o2o::o2o)]
#[from_owned(i64| repeat(), return Self(@.to_string()))]
#[owned_try_into(i64, std::num::ParseIntError| repeat(), return Ok(@.0.parse()?))]
#[from_owned(i32)]
#[owned_try_into(i32, std::num::ParseIntError)]
#[from_owned(i16| skip_repeat, return Self("321".into()))]
#[owned_try_into(i16, std::num::ParseIntError)]
#[from_owned(i8)]
#[owned_try_into(i8, std::num::ParseIntError| skip_repeat, return Ok(123))]
#[from_owned(f32| stop_repeat, repeat(), return Self("654".into()))]
#[owned_try_into(f32, std::num::ParseFloatError| stop_repeat, repeat(), return Ok(456.0))]
#[from_owned(f64)]
#[owned_try_into(f64, std::num::ParseFloatError)]
struct Wrapper(String);

#[test]
fn from() {
    let a: i64 = 111;
    let wrapper = Wrapper::from(a);
    assert_eq!("111", wrapper.0);

    let a: i32 = 222;
    let wrapper = Wrapper::from(a);
    assert_eq!("222", wrapper.0);

    let a: i16 = 333;
    let wrapper = Wrapper::from(a);
    assert_eq!("321", wrapper.0);

    let a: i8 = 44;
    let wrapper = Wrapper::from(a);
    assert_eq!("44", wrapper.0);

    let a: f32 = 555.0;
    let wrapper = Wrapper::from(a);
    assert_eq!("654", wrapper.0);

    let a: f64 = 666.0;
    let wrapper = Wrapper::from(a);
    assert_eq!("654", wrapper.0);
}

#[test]
fn into() {
    let wrapper = Wrapper("111".into());
    let b: i64 = wrapper.try_into().unwrap();
    assert_eq!(111, b);

    let wrapper = Wrapper("222".into());
    let b: i32 = wrapper.try_into().unwrap();
    assert_eq!(222, b);

    let wrapper = Wrapper("333".into());
    let b: i16 = wrapper.try_into().unwrap();
    assert_eq!(333, b);

    let wrapper = Wrapper("444".into());
    let b: i8 = wrapper.try_into().unwrap();
    assert_eq!(123, b);

    let wrapper = Wrapper("555".into());
    let b: f32 = wrapper.try_into().unwrap();
    assert_eq!(456.0, b);

    let wrapper = Wrapper("666".into());
    let b: f64 = wrapper.try_into().unwrap();
    assert_eq!(456.0, b);
}
