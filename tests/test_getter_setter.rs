use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::types::IntoPyDict;
use std::isize;

mod common;

#[pyclass]
struct ClassWithProperties {
    num: i32,
}

#[pymethods]
impl ClassWithProperties {
    fn get_num(&self) -> PyResult<i32> {
        Ok(self.num)
    }

    #[getter(DATA)]
    /// a getter for data
    fn get_data(&self) -> PyResult<i32> {
        Ok(self.num)
    }
    #[setter(DATA)]
    fn set_data(&mut self, value: i32) -> PyResult<()> {
        self.num = value;
        Ok(())
    }

    #[getter]
    /// a getter with a type un-wrapped by PyResult
    fn get_unwrapped(&self) -> i32 {
        self.num
    }
    #[setter]
    fn set_unwrapped(&mut self, value: i32) {
        self.num = value;
    }
}

#[test]
fn class_with_properties() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Py::new(py, ClassWithProperties { num: 10 }).unwrap();

    py_run!(py, inst, "assert inst.get_num() == 10");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");
    py_run!(py, inst, "inst.DATA = 20");
    py_run!(py, inst, "assert inst.get_num() == 20");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");

    py_run!(py, inst, "assert inst.get_num() == inst.unwrapped == 20");
    py_run!(py, inst, "inst.unwrapped = 42");
    py_run!(py, inst, "assert inst.get_num() == inst.unwrapped == 42");

    let d = [("C", py.get_type::<ClassWithProperties>())].into_py_dict(py);
    py.run(
        "assert C.DATA.__doc__ == 'a getter for data'",
        None,
        Some(d),
    )
    .unwrap();
}

#[pyclass]
struct GetterSetter {
    #[pyo3(get, set)]
    num: i32,
    #[pyo3(get, set)]
    text: String,
}

#[pymethods]
impl GetterSetter {
    fn get_num2(&self) -> PyResult<i32> {
        Ok(self.num)
    }
}

#[test]
fn getter_setter_autogen() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Py::new(
        py,
        GetterSetter {
            num: 10,
            text: "Hello".to_string(),
        },
    )
    .unwrap();

    py_run!(py, inst, "assert inst.num == 10");
    py_run!(py, inst, "inst.num = 20; assert inst.num == 20");
    py_run!(
        py,
        inst,
        "assert inst.text == 'Hello'; inst.text = 'There'; assert inst.text == 'There'"
    );
}
