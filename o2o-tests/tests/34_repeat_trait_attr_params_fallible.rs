use std::fmt::Debug;

#[derive(Debug)]
struct Thing1;

#[derive(Debug)]
struct Thing2;

#[derive(Debug)]
struct Thing3;

#[derive(Debug)]
struct Thing4;

#[derive(Debug)]
struct Thing5;

#[derive(Debug)]
struct Thing6;

#[derive(Debug)]
struct Thing7;

#[derive(o2o::o2o)]
#[try_from_owned(Thing1, String| repeat(), return Ok(Self(format!("Hi, I'm {:?}", @))))]
#[try_from_owned(Thing2, String)]
#[try_from_owned(Thing3, String| skip_repeat, return Ok(Self("Custom Thing3 message".into())))]
#[try_from_owned(Thing4, String)]
#[try_from_owned(Thing5, String)]
#[try_from_owned(Thing6, String| stop_repeat, repeat(), return Ok(Self("stuff happens".into())))]
#[try_from_owned(Thing7, String)]
struct Wrapper(String);

#[derive(o2o::o2o)]
#[try_from_owned(Thing1, String| repeat(quick_return), vars(msg: {format!("Hi, I'm {:?}", @)}), return Ok(Self(msg)))]
#[try_from_owned(Thing2, String| vars(msg: {"test123".into()}))]
#[try_from_owned(Thing3, String| skip_repeat, return Ok(Self("Custom Thing3 message".into())))]
#[try_from_owned(Thing4, String| vars(msg: {"test456".into()}))]
#[try_from_owned(Thing5, String| vars(msg: {"test789".into()}))]
#[try_from_owned(Thing6, String| stop_repeat, repeat(), return Ok(Self("stuff happens".into())))]
#[try_from_owned(Thing7, String)]
struct Wrapper2(String);

#[test]
fn test() {
    let thing = Thing1;
    let wrapper: Wrapper = thing.try_into().unwrap();
    assert_eq!("Hi, I'm Thing1", wrapper.0);

    let thing = Thing2;
    let wrapper: Wrapper = thing.try_into().unwrap();
    assert_eq!("Hi, I'm Thing2", wrapper.0);

    let thing = Thing3;
    let wrapper: Wrapper = thing.try_into().unwrap();
    assert_eq!("Custom Thing3 message", wrapper.0);

    let thing = Thing4;
    let wrapper: Wrapper = thing.try_into().unwrap();
    assert_eq!("Hi, I'm Thing4", wrapper.0);

    let thing = Thing5;
    let wrapper: Wrapper = thing.try_into().unwrap();
    assert_eq!("Hi, I'm Thing5", wrapper.0);

    let thing = Thing6;
    let wrapper: Wrapper = thing.try_into().unwrap();
    assert_eq!("stuff happens", wrapper.0);

    let thing = Thing7;
    let wrapper: Wrapper = thing.try_into().unwrap();
    assert_eq!("stuff happens", wrapper.0);
}

#[test]
fn test2() {
    let thing = Thing1;
    let wrapper: Wrapper2 = thing.try_into().unwrap();
    assert_eq!("Hi, I'm Thing1", wrapper.0);

    let thing = Thing2;
    let wrapper: Wrapper2 = thing.try_into().unwrap();
    assert_eq!("test123", wrapper.0);

    let thing = Thing3;
    let wrapper: Wrapper2 = thing.try_into().unwrap();
    assert_eq!("Custom Thing3 message", wrapper.0);

    let thing = Thing4;
    let wrapper: Wrapper2 = thing.try_into().unwrap();
    assert_eq!("test456", wrapper.0);

    let thing = Thing5;
    let wrapper: Wrapper2 = thing.try_into().unwrap();
    assert_eq!("test789", wrapper.0);

    let thing = Thing6;
    let wrapper: Wrapper2 = thing.try_into().unwrap();
    assert_eq!("stuff happens", wrapper.0);

    let thing = Thing7;
    let wrapper: Wrapper2 = thing.try_into().unwrap();
    assert_eq!("stuff happens", wrapper.0);
}