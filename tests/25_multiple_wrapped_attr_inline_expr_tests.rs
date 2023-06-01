use o2o::o2o;

#[derive(Clone, o2o)]
#[map(StuffWrapper)]
#[map(StuffWrapper3 as ())]
#[o2o(wrapped(StuffWrapper| payload, ~.clone()))]
#[o2o(wrapped(StuffWrapper3| 0, ~.clone()))]
struct Stuff{thing: i32}

#[derive(Clone, o2o)]
#[map(StuffWrapper2 as {})]
#[map(StuffWrapper4)]
#[o2o(wrapped(StuffWrapper2| payload, ~.clone()))]
#[o2o(wrapped(StuffWrapper4| 0, ~.clone()))]
struct TupleStuff(i32);

struct StuffWrapper {
    payload: Stuff,
}

struct StuffWrapper2 {
    payload: TupleStuff,
}

struct StuffWrapper3(Stuff);

struct StuffWrapper4(TupleStuff);

#[test]
fn wrappee2wrapper() {
    let payload = TupleStuff(123);

    let wrapper: StuffWrapper2 = payload.into();

    assert_eq!(123, wrapper.payload.0);
}

#[test]
fn wrappee2wrapper_ref() {
    let payload = &TupleStuff(123);

    let wrapper: StuffWrapper2 = payload.into();

    assert_eq!(payload.0, wrapper.payload.0);
}

#[test]
fn wrapper2wrappee() {
    let wrapper = StuffWrapper2 { payload: TupleStuff(123) };

    let payload: TupleStuff = wrapper.into();

    assert_eq!(123, payload.0);
}

#[test]
fn wrapper2wrappee_ref() {
    let wrapper = &StuffWrapper2 { payload: TupleStuff(123) };

    let payload: TupleStuff = wrapper.into();

    assert_eq!(wrapper.payload.0, payload.0);
}

#[test]
fn wrappee2wrapper_2() {
    let payload = Stuff {thing: 123};

    let wrapper: StuffWrapper = payload.into();

    assert_eq!(123, wrapper.payload.thing);
}

#[test]
fn wrappee2wrapper_ref_2() {
    let payload = &Stuff {thing: 123};

    let wrapper: StuffWrapper = payload.into();

    assert_eq!(payload.thing, wrapper.payload.thing);
}

#[test]
fn wrapper2wrappee_2() {
    let wrapper = StuffWrapper { payload: Stuff { thing: 123 }};

    let payload: Stuff = wrapper.into();

    assert_eq!(123, payload.thing);
}

#[test]
fn wrapper2wrappee_ref_2() {
    let wrapper = &StuffWrapper { payload: Stuff { thing: 123 }};

    let payload: Stuff = wrapper.into();

    assert_eq!(wrapper.payload.thing, payload.thing);
}

#[test]
fn wrappee2wrapper_3() {
    let payload = TupleStuff(123);

    let wrapper: StuffWrapper4 = payload.into();

    assert_eq!(123, wrapper.0.0);
}

#[test]
fn wrappee2wrapper_ref_3() {
    let payload = &TupleStuff(123);

    let wrapper: StuffWrapper4 = payload.into();

    assert_eq!(payload.0, wrapper.0.0);
}

#[test]
fn wrapper2wrappee_3() {
    let wrapper = StuffWrapper4(TupleStuff(123));

    let payload: TupleStuff = wrapper.into();

    assert_eq!(123, payload.0);
}

#[test]
fn wrapper2wrappee_ref_3() {
    let wrapper = &StuffWrapper4(TupleStuff(123));

    let payload: TupleStuff = wrapper.into();

    assert_eq!(wrapper.0.0, payload.0);
}

#[test]
fn wrappee2wrapper_4() {
    let payload = Stuff{thing: 123};

    let wrapper: StuffWrapper3 = payload.into();

    assert_eq!(123, wrapper.0.thing);
}

#[test]
fn wrappee2wrapper_ref_4() {
    let payload = &Stuff{thing: 123};

    let wrapper: StuffWrapper3 = payload.into();

    assert_eq!(payload.thing, wrapper.0.thing);
}

#[test]
fn wrapper2wrappee_4() {
    let wrapper = StuffWrapper3(Stuff { thing: 123 });

    let payload: Stuff = wrapper.into();

    assert_eq!(123, payload.thing);
}

#[test]
fn wrapper2wrappee_ref_4() {
    let wrapper = &StuffWrapper3(Stuff { thing: 123 });

    let payload: Stuff = wrapper.into();

    assert_eq!(wrapper.0.thing, payload.thing);
}