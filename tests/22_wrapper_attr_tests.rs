use o2o::o2o;

#[derive(o2o)]
#[map_owned(Vec<u8>)]
struct PayloadWrapper {
    #[o2o(wrapper)]
    payload: Vec<u8>,
}

#[derive(o2o)]
#[map(String)]
struct StringWrapper {
    #[o2o(wrapper(~.clone()))]
    str: String,
    #[ghost({None})]
    opt: Option<usize>
}

#[test]
fn wrappee2wrapper() {
    let payload = vec![42, 123];

    let wrapper: PayloadWrapper = payload.into();

    assert_eq!(vec![42, 123], wrapper.payload);
}

#[test]
fn wrapper2wrappee() {
    let wrapper = PayloadWrapper {
        payload: vec![42, 123]
    };

    let payload: Vec<u8> = wrapper.into();

    assert_eq!(vec![42, 123], payload)
}

#[test]
fn wrappee2wrapper_2() {
    let str = String::from("Test");

    let wrapper: StringWrapper = str.into();

    assert_eq!("Test", wrapper.str);
    assert_eq!(None, wrapper.opt);
}

#[test]
fn wrapper2wrappee_2() {
    let wrapper = StringWrapper {
        str: "Test".into(),
        opt: None
    };

    let str: String = wrapper.into();

    assert_eq!("Test", str)
}

#[test]
fn wrappee2wrapper_ref_2() {
    let str = &String::from("Test");

    let wrapper: StringWrapper = str.into();

    assert_eq!(str, &wrapper.str);
    assert_eq!(None, wrapper.opt);
}

#[test]
fn wrapper2wrappee_ref_2() {
    let wrapper = &StringWrapper {
        str: "Test".into(),
        opt: None
    };

    let str: String = wrapper.into();

    assert_eq!(wrapper.str, str)
}