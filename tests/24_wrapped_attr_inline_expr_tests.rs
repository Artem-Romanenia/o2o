use o2o::o2o;

#[derive(Clone, o2o)]
#[map(StuffWrapper as {})]
#[o2o(wrapped(payload, ~.clone()))]
struct Stuff(i32);

struct StuffWrapper {
    payload: Stuff,
}

#[derive(Clone, o2o)]
#[map(StuffWrapper2)]
#[o2o(wrapped(payload, ~.clone()))]
struct Stuff2{
    thing: i32
}

struct StuffWrapper2 {
    payload: Stuff2,
}

#[derive(Clone, o2o)]
#[map(StuffWrapper3)]
#[o2o(wrapped(0, ~.clone()))]
struct Stuff3(i32);

struct StuffWrapper3(Stuff3);

#[derive(Clone, o2o)]
#[map(StuffWrapper4 as ())]
#[o2o(wrapped(0, ~.clone()))]
struct Stuff4{
    thing: i32
}

struct StuffWrapper4(Stuff4);

#[test]
fn wrappee2wrapper() {
    let payload = Stuff(123);

    let wrapper: StuffWrapper = payload.into();

    assert_eq!(123, wrapper.payload.0);
}

#[test]
fn wrappee2wrapper_ref() {
    let payload = &Stuff(123);

    let wrapper: StuffWrapper = payload.into();

    assert_eq!(payload.0, wrapper.payload.0);
}

#[test]
fn wrapper2wrappee() {
    let wrapper = StuffWrapper { payload: Stuff(123) };

    let payload: Stuff = wrapper.into();

    assert_eq!(123, payload.0);
}

#[test]
fn wrapper2wrappee_ref() {
    let wrapper = &StuffWrapper { payload: Stuff(123) };

    let payload: Stuff = wrapper.into();

    assert_eq!(wrapper.payload.0, payload.0);
}

#[test]
fn wrappee2wrapper_2() {
    let payload = Stuff2 {thing: 123};

    let wrapper: StuffWrapper2 = payload.into();

    assert_eq!(123, wrapper.payload.thing);
}

#[test]
fn wrappee2wrapper_ref_2() {
    let payload = &Stuff2 {thing: 123};

    let wrapper: StuffWrapper2 = payload.into();

    assert_eq!(payload.thing, wrapper.payload.thing);
}

#[test]
fn wrapper2wrappee_2() {
    let wrapper = StuffWrapper2 { payload: Stuff2 { thing: 123 }};

    let payload: Stuff2 = wrapper.into();

    assert_eq!(123, payload.thing);
}

#[test]
fn wrapper2wrappee_ref_2() {
    let wrapper = &StuffWrapper2 { payload: Stuff2 { thing: 123 }};

    let payload: Stuff2 = wrapper.into();

    assert_eq!(wrapper.payload.thing, payload.thing);
}

#[test]
fn wrappee2wrapper_3() {
    let payload = Stuff3(123);

    let wrapper: StuffWrapper3 = payload.into();

    assert_eq!(123, wrapper.0.0);
}

#[test]
fn wrappee2wrapper_ref_3() {
    let payload = &Stuff3(123);

    let wrapper: StuffWrapper3 = payload.into();

    assert_eq!(payload.0, wrapper.0.0);
}

#[test]
fn wrapper2wrappee_3() {
    let wrapper = StuffWrapper3(Stuff3(123));

    let payload: Stuff3 = wrapper.into();

    assert_eq!(123, payload.0);
}

#[test]
fn wrapper2wrappee_ref_3() {
    let wrapper = &StuffWrapper3(Stuff3(123));

    let payload: Stuff3 = wrapper.into();

    assert_eq!(wrapper.0.0, payload.0);
}

#[test]
fn wrappee2wrapper_4() {
    let payload = Stuff4{thing: 123};

    let wrapper: StuffWrapper4 = payload.into();

    assert_eq!(123, wrapper.0.thing);
}

#[test]
fn wrappee2wrapper_ref_4() {
    let payload = &Stuff4{thing: 123};

    let wrapper: StuffWrapper4 = payload.into();

    assert_eq!(payload.thing, wrapper.0.thing);
}

#[test]
fn wrapper2wrappee_4() {
    let wrapper = StuffWrapper4(Stuff4 { thing: 123 });

    let payload: Stuff4 = wrapper.into();

    assert_eq!(123, payload.thing);
}

#[test]
fn wrapper2wrappee_ref_4() {
    let wrapper = &StuffWrapper4(Stuff4 { thing: 123 });

    let payload: Stuff4 = wrapper.into();

    assert_eq!(wrapper.0.thing, payload.thing);
}