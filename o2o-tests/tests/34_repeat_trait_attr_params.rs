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
#[from_owned(Thing1| repeat(), return Self(format!("Hi, I'm {:?}", @)))]
#[from_owned(Thing2)]
#[from_owned(Thing3| skip_repeat, return Self("Custom Thing3 message".into()))]
#[from_owned(Thing4)]
#[from_owned(Thing5)]
#[from_owned(Thing6| stop_repeat, repeat(), return Self("stuff happens".into()))]
#[from_owned(Thing7)]
struct Wrapper(String);

#[derive(o2o::o2o)]
#[from_owned(Thing1| repeat(quick_return), vars(msg: {format!("Hi, I'm {:?}", @)}), return Self(msg))]
#[from_owned(Thing2| vars(msg: {"test123".into()}))]
#[from_owned(Thing3| skip_repeat, return Self("Custom Thing3 message".into()))]
#[from_owned(Thing4| vars(msg: {"test456".into()}))]
#[from_owned(Thing5| vars(msg: {"test789".into()}))]
#[from_owned(Thing6| stop_repeat, repeat(), return Self("stuff happens".into()))]
#[from_owned(Thing7)]
struct Wrapper2(String);

#[test]
fn test() {
    let thing = Thing1;
    let wrapper: Wrapper = thing.into();
    assert_eq!("Hi, I'm Thing1", wrapper.0);

    let thing = Thing2;
    let wrapper: Wrapper = thing.into();
    assert_eq!("Hi, I'm Thing2", wrapper.0);

    let thing = Thing3;
    let wrapper: Wrapper = thing.into();
    assert_eq!("Custom Thing3 message", wrapper.0);

    let thing = Thing4;
    let wrapper: Wrapper = thing.into();
    assert_eq!("Hi, I'm Thing4", wrapper.0);

    let thing = Thing5;
    let wrapper: Wrapper = thing.into();
    assert_eq!("Hi, I'm Thing5", wrapper.0);

    let thing = Thing6;
    let wrapper: Wrapper = thing.into();
    assert_eq!("stuff happens", wrapper.0);

    let thing = Thing7;
    let wrapper: Wrapper = thing.into();
    assert_eq!("stuff happens", wrapper.0);
}

#[test]
fn test2() {
    let thing = Thing1;
    let wrapper: Wrapper2 = thing.into();
    assert_eq!("Hi, I'm Thing1", wrapper.0);

    let thing = Thing2;
    let wrapper: Wrapper2 = thing.into();
    assert_eq!("test123", wrapper.0);

    let thing = Thing3;
    let wrapper: Wrapper2 = thing.into();
    assert_eq!("Custom Thing3 message", wrapper.0);

    let thing = Thing4;
    let wrapper: Wrapper2 = thing.into();
    assert_eq!("test456", wrapper.0);

    let thing = Thing5;
    let wrapper: Wrapper2 = thing.into();
    assert_eq!("test789", wrapper.0);

    let thing = Thing6;
    let wrapper: Wrapper2 = thing.into();
    assert_eq!("stuff happens", wrapper.0);

    let thing = Thing7;
    let wrapper: Wrapper2 = thing.into();
    assert_eq!("stuff happens", wrapper.0);
}