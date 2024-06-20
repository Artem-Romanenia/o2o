use o2o::o2o;
use o2o::traits::TryIntoExisting;

#[derive(o2o)]
#[o2o(owned_try_into(i32, String| vars(hrs: {@.hours as i32}, mns: {@.minutes as i32}, scs: {@.seconds as i32}),
    return Ok(hrs * 3600 + mns * 60 + scs)))]
struct Time {
    hours: i8,
    minutes: i8,
    seconds: i8,
}

#[derive(o2o)]
#[try_from(Time, String| vars(hrs: {@.hours as i32}, mns: {@.minutes as i32}, scs: {@.seconds as i32}),
    return { Ok(TotalTime {total_seconds: hrs * 3600 + mns * 60 + scs}) })]
#[try_into(String, String| return Ok(@.total_seconds.to_string()))]
#[try_into_existing(String, String| return @.total_seconds.to_string())]
struct TotalTime {
    total_seconds: i32
}

#[test]
fn time2i() {
    let time = Time {
        hours: 2,
        minutes: 10,
        seconds: 15
    };

    let i: i32 = time.try_into().unwrap();

    assert_eq!(7815, i);
}

#[test]
fn named2named() {
    let time = Time {
        hours: 2,
        minutes: 10,
        seconds: 15
    };

    let total: TotalTime = time.try_into().unwrap();

    assert_eq!(7815, total.total_seconds);
}

#[test]
fn named2named_ref() {
    let time = &Time {
        hours: 2,
        minutes: 10,
        seconds: 15
    };

    let total: TotalTime = time.try_into().unwrap();

    let hrs = time.hours as i32;
    let mns = time.minutes as i32;
    let scs = time.seconds as i32;
    assert_eq!(hrs*3600+mns*60+scs, total.total_seconds);
}

#[test]
fn time2string() {
    let total_time = TotalTime {
        total_seconds: 123
    };

    let str: String = total_time.try_into().unwrap();

    assert_eq!("123", str);
}

#[test]
fn time2string_ref() {
    let total_time = &TotalTime {
        total_seconds: 123
    };

    let str: String = total_time.try_into().unwrap();

    assert_eq!("123", str);
}

#[test]
fn existing_time2string() {
    let total_time = TotalTime {
        total_seconds: 123
    };

    let mut str = String::new();
    total_time.try_into_existing(&mut str).unwrap();

    assert_eq!("123", str);
}

#[test]
fn existing_time2string_ref() {
    let total_time = &TotalTime {
        total_seconds: 123
    };

    let mut str = String::new();
    total_time.try_into_existing(&mut str).unwrap();

    assert_eq!("123", str);
}