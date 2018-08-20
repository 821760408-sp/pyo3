#![feature(concat_idents)]

extern crate pyo3;

use pyo3::prelude::*;

use pyo3::ffi::*;

fn _get_subclasses<'p>(
    py: &'p Python,
    py_type: &str,
    args: &str,
) -> PyResult<(&'p PyObjectRef, &'p PyObjectRef, &'p PyObjectRef)> {
    // Import the class from Python and create some subclasses
    let datetime = py.import("datetime")?;

    let locals = PyDict::new(*py);
    locals.set_item(py_type, datetime.get(py_type)?).unwrap();

    let make_subclass_py = format!("class Subklass({}):\n    pass", py_type);

    let make_sub_subclass_py = "class SubSubklass(Subklass):\n    pass";

    py.run(&make_subclass_py, None, Some(&locals))?;
    py.run(&make_sub_subclass_py, None, Some(&locals))?;

    // Construct an instance of the base class
    let obj = py.eval(&format!("{}({})", py_type, args), None, Some(&locals))?;

    // Construct an instance of the subclass
    let sub_obj = py.eval(&format!("Subklass({})", args), None, Some(&locals))?;

    // Construct an instance of the sub-subclass
    let sub_sub_obj = py.eval(&format!("SubSubklass({})", args), None, Some(&locals))?;

    Ok((obj, sub_obj, sub_sub_obj))
}

macro_rules! assert_check_exact {
    ($check_func:ident, $obj: expr) => {
        unsafe {
            assert!($check_func(($obj).as_ptr()) != 0);
            assert!(concat_idents!($check_func, Exact)(($obj).as_ptr()) != 0);
        }
    };
}

macro_rules! assert_check_only {
    ($check_func:ident, $obj: expr) => {
        unsafe {
            assert!($check_func(($obj).as_ptr()) != 0);
            assert!(concat_idents!($check_func, Exact)(($obj).as_ptr()) == 0);
        }
    };
}

#[test]
fn test_date_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py, "date", "2018, 1, 1").unwrap();

    assert_check_exact!(PyDate_Check, obj);
    assert_check_only!(PyDate_Check, sub_obj);
    assert_check_only!(PyDate_Check, sub_sub_obj);
}

#[test]
fn test_time_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py, "time", "12, 30, 15").unwrap();

    assert_check_exact!(PyTime_Check, obj);
    assert_check_only!(PyTime_Check, sub_obj);
    assert_check_only!(PyTime_Check, sub_sub_obj);
}

#[test]
fn test_datetime_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) =
        _get_subclasses(&py, "datetime", "2018, 1, 1, 13, 30, 15").unwrap();

    assert_check_only!(PyDate_Check, obj);
    assert_check_exact!(PyDateTime_Check, obj);
    assert_check_only!(PyDateTime_Check, sub_obj);
    assert_check_only!(PyDateTime_Check, sub_sub_obj);
}

#[test]
fn test_delta_check() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let (obj, sub_obj, sub_sub_obj) = _get_subclasses(&py, "timedelta", "1, -3").unwrap();

    assert_check_exact!(PyDelta_Check, obj);
    assert_check_only!(PyDelta_Check, sub_obj);
    assert_check_only!(PyDelta_Check, sub_sub_obj);
}

#[test]
#[cfg(Py_3)]
fn test_datetime_utc() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let datetime = py.import("datetime").map_err(|e| e.print(py)).unwrap();
    let timezone = datetime.get("timezone").unwrap();
    let utc = timezone.getattr("utc").unwrap().to_object(py);

    let dt = PyDateTime::new(py, 2018, 1, 1, 0, 0, 0, 0, Some(&utc)).unwrap();

    let locals = PyDict::new(py);
    locals.set_item("dt", dt).unwrap();

    let offset: f32 = py
        .eval("dt.utcoffset().total_seconds()", None, Some(locals))
        .unwrap()
        .extract()
        .unwrap();
    assert_eq!(offset, 0f32);
}

#[cfg(Py_3_6)]
#[test]
fn test_pydate_out_of_bounds() {
    // This test is an XFAIL on Python < 3.6 until bounds checking is implemented
    let gil = Python::acquire_gil();
    let py = gil.python();
    let vals = [
        (2147484672u32, 1, 1),
        (2018, 2147484672u32, 1),
        (2018, 1, 2147484672u32),
    ];

    for val in vals.into_iter() {
        let (year, month, day) = val;
        let dt = PyDate::new(py, *year, *month, *day);
        let msg = format!("Should have raised an error: {:#?}", val);
        match dt {
            Ok(_) => assert!(false, msg),
            Err(_) => assert!(true),
        }
    }
}

#[cfg(Py_3_6)]
#[test]
fn test_pytime_out_of_bounds() {
    // This test is an XFAIL on Python < 3.6 until bounds checking is implemented
    let gil = Python::acquire_gil();
    let py = gil.python();
    let vals = [
        (2147484672u32, 0, 0, 0),
        (0, 2147484672u32, 0, 0),
        (0, 0, 2147484672u32, 0),
        (0, 0, 0, 2147484672u32),
    ];

    for val in vals.into_iter() {
        let (hour, minute, second, microsecond) = val;
        let dt = PyTime::new(py, *hour, *minute, *second, *microsecond, None);
        let msg = format!("Should have raised an error: {:#?}", val);
        match dt {
            Ok(_) => assert!(false, msg),
            Err(_) => assert!(true),
        }
    }
}

#[cfg(Py_3_6)]
#[test]
fn test_pydatetime_out_of_bounds() {
    // This test is an XFAIL on Python < 3.6 until bounds checking is implemented
    let gil = Python::acquire_gil();
    let py = gil.python();
    let vals = [
        (2147484672u32, 1, 1, 0, 0, 0, 0),
        (2018, 2147484672u32, 1, 0, 0, 0, 0),
        (2018, 1, 2147484672u32, 0, 0, 0, 0),
        (2018, 1, 1, 2147484672u32, 0, 0, 0),
        (2018, 1, 1, 0, 2147484672u32, 0, 0),
        (2018, 1, 1, 0, 0, 2147484672u32, 0),
        (2018, 1, 1, 0, 0, 0, 2147484672u32),
    ];

    for val in vals.into_iter() {
        let (year, month, day, hour, minute, second, microsecond) = val;
        let dt = PyDateTime::new(
            py,
            *year,
            *month,
            *day,
            *hour,
            *minute,
            *second,
            *microsecond,
            None,
        );
        let msg = format!("Should have raised an error: {:#?}", val);
        match dt {
            Ok(_) => assert!(false, msg),
            Err(_) => assert!(true),
        }
    }
}
