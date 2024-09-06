use o2o::traits::IntoExisting;

#[test]
fn struct2unit() {
    struct A;

    #[derive(o2o::o2o)]
    #[map(A as Unit)]
    #[into_existing(A as Unit)]
    struct B {
        #[ghost({123})]
        x: i32,
    }

    let a = A;
    let b: B = a.into();
    assert_eq!(123, b.x);

    let b = B { x: 111 };
    let a: A = b.into();
    assert!(matches!(a, A));

    let a = &A;
    let b: B = a.into();
    assert_eq!(123, b.x);

    let b = &B { x: 111 };
    let a: A = b.into();
    assert!(matches!(a, A));

    let mut a = A;
    let b = B { x: 111 };
    b.into_existing(&mut a);

    let mut a = A;
    let b = &B { x: 111 };
    b.into_existing(&mut a);
}

#[test]
fn tuple2unit() {
    struct A;

    #[derive(o2o::o2o)]
    #[map(A as Unit)]
    #[into_existing(A as Unit)]
    struct B(#[ghost({123})] i32);

    let a = A;
    let b: B = a.into();
    assert_eq!(123, b.0);

    let b = B(111);
    let a: A = b.into();
    assert!(matches!(a, A));

    let a = &A;
    let b: B = a.into();
    assert_eq!(123, b.0);

    let b = &B(111);
    let a: A = b.into();
    assert!(matches!(a, A));

    let mut a = A;
    let b = B(111);
    b.into_existing(&mut a);

    let mut a = A;
    let b = &B(111);
    b.into_existing(&mut a);
}

#[test]
fn unit2struct() {
    #[derive(o2o::o2o)]
    #[map(B as {})]
    #[into_existing(B as {})]
    #[ghosts(x: {123})]
    struct A;

    struct B {
        x: i32,
    }

    let a = A;
    let b: B = a.into();
    assert_eq!(123, b.x);

    let b = B { x: 111 };
    let a: A = b.into();
    assert!(matches!(a, A));

    let a = &A;
    let b: B = a.into();
    assert_eq!(123, b.x);

    let b = &B { x: 111 };
    let a: A = b.into();
    assert!(matches!(a, A));

    let a = A;
    let mut b = B { x: 111 };
    a.into_existing(&mut b);
    assert_eq!(123, b.x);

    let a = &A;
    let mut b = B { x: 111 };
    a.into_existing(&mut b);
    assert_eq!(123, b.x);
}

#[test]
fn unit2tuple() {
    #[derive(o2o::o2o)]
    #[map(B as ())]
    #[into_existing(B as ())]
    #[ghosts(0: {123})]
    struct A;

    struct B(i32);

    let a = A;
    let b: B = a.into();
    assert_eq!(123, b.0);

    let b = B(111);
    let a: A = b.into();
    assert!(matches!(a, A));

    let a = &A;
    let b: B = a.into();
    assert_eq!(123, b.0);

    let b = &B(111);
    let a: A = b.into();
    assert!(matches!(a, A));

    let a = A;
    let mut b = B(111);
    a.into_existing(&mut b);
    assert_eq!(123, b.0);

    let a = &A;
    let mut b = B(111);
    a.into_existing(&mut b);
    assert_eq!(123, b.0);
}

#[test]
fn struct2unit_no_ghost() {
    struct A;

    #[derive(o2o::o2o)]
    #[into(A as Unit)]
    #[into_existing(A as Unit)]
    struct B {
        _x: i32,
    }

    let b = B { _x: 111 };
    let a: A = b.into();
    assert!(matches!(a, A));

    let b = &B { _x: 111 };
    let a: A = b.into();
    assert!(matches!(a, A));

    let mut a = A;
    let b = B { _x: 111 };
    b.into_existing(&mut a);

    let mut a = A;
    let b = &B { _x: 111 };
    b.into_existing(&mut a);
}

#[test]
fn tuple2unit_no_ghost() {
    struct A;

    #[derive(o2o::o2o)]
    #[into(A as Unit)]
    #[into_existing(A as Unit)]
    struct B(#[allow(dead_code)] i32);

    let b = B(111);
    let a: A = b.into();
    assert!(matches!(a, A));

    let b = &B(111);
    let a: A = b.into();
    assert!(matches!(a, A));

    let mut a = A;
    let b = B(111);
    b.into_existing(&mut a);

    let mut a = A;
    let b = &B(111);
    b.into_existing(&mut a);
}

#[test]
fn unit2struct_no_ghost() {
    #[derive(o2o::o2o)]
    #[from(B)]
    struct A;

    struct B {
        _x: i32,
    }

    let b = B { _x: 111 };
    let a: A = b.into();
    assert!(matches!(a, A));

    let b = &B { _x: 111 };
    let a: A = b.into();
    assert!(matches!(a, A));
}

#[test]
fn unit2tuple_no_ghost() {
    #[derive(o2o::o2o)]
    #[from(B)]
    struct A;

    struct B(#[allow(dead_code)] i32);

    let b = B(111);
    let a: A = b.into();
    assert!(matches!(a, A));

    let b = &B(111);
    let a: A = b.into();
    assert!(matches!(a, A));
}
