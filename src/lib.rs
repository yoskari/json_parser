extern crate cpython;
use cpython::{
    PyResult,
    Python,
    py_module_initializer,
    py_fn,
    PyObject,
    ToPyObject,
    PyErr,
    PyDict,
    PyNone,
    PyString,
    PyBool,
    PyList,
    PyInt,
    PyFloat,
    PythonObject,
};

use std::{
    collections::HashMap,
    borrow::Borrow,
};

const NUMERIC: [char; 12] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', '-'];

const BOOLCHARS: [char; 8] = ['t', 'r', 'u', 'e', 'f', 'a', 's', 'e'];

const NULLCHARS: [char; 3] = ['n', 'u', 'l'];

#[derive(Clone, Debug)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Box<Json>>),
    Object(HashMap<String, Box<Json>>)
}

py_module_initializer!(json_parser, |py, m| {
    m.add(py, "__doc__", "This is a shitty json module implemented in Rust")?;
    m.add(py, "dumps", py_fn!(py, dumps(obj: PyObject)))?;
    m.add(py, "loads", py_fn!(py, loads(s: String)))?;
    Ok(())
});

fn dumps(py: Python, obj: PyObject) -> PyResult<String> {
    let json = python_to_json(py, obj)?;
    Ok(json_to_string(json, 0, false))
}

fn loads(py: Python, s: String) -> PyResult<PyObject> {
    let json = string_to_json(s);
    Ok(json_to_python(json, py))
}

fn unbox<T>(value: Box<T>) -> T {
    *value
}

fn string_to_json(s: String) -> Json {
    fn add_to_previous(stack: &mut Vec<Json>, key: Option<String>, item: Json) {
        if !stack.is_empty() {
            let l = stack.len();
            match &mut stack[l - 1] {
                Json::Object(obj) => {
                    obj.insert(key.unwrap_or("ERROR".to_string()), Box::new(item));
                },
                Json::Array(arr) => {
                    arr.push(Box::new(item));
                },
                t => panic!("INVALID TOP LEVEL STRUCTURE {:?}", t),
            }
        }
    }

    fn flush_buffer(stack: &mut Vec<Json>, buffer: &mut String, current_type: &mut i32,
        key_stack: &mut Vec<String>) 
    {
        let item: Json;
        //let key: Option<String>;
        if current_type == &1 {
            item = Json::Number(buffer.parse::<f64>().unwrap());
        } else if current_type == &2 {
            item = Json::Bool(buffer.parse::<bool>().unwrap());
        } else if current_type == &3 {
            item = Json::Null;
        } else {
            item = Json::String(buffer.clone());
        }
        match stack.last().unwrap() {
            Json::Array(_) => {
                add_to_previous(stack, None, item);
            },
            Json::Object(_) => {
                match key_stack.pop() {
                    Some(k) => add_to_previous(stack, Some(k), item),
                    None => {},
                };
            },
            t => {
                panic!("INVALID TOP LEVEL STRUCTURE: {:?}", t);
            },
        }
        buffer.clear();
        *current_type = 0;
    }
    //println!("{}", s);
    let mut stack = Vec::<Json>::new();
    let mut key_stack = Vec::<String>::new();
    let mut buffer = String::new();

    let mut last_value = Json::Null;
    let mut recording = false;
    let mut current_type = 0; // 0 = everything else, 1 = numeric, 2 = boolean, 3 = null

    for c in s.chars() {
        match c {
            '{' => stack.push(Json::Object(HashMap::new())),
            '}' => {
                //println!("key stack: {:?}", key_stack);
                //flush_buffer(&mut stack, &mut buffer, &mut current_type, &mut key_stack);
                let item = stack.pop().unwrap();
                last_value = item.clone();
                add_to_previous(&mut stack, key_stack.pop(), item);
            },
            '[' => stack.push(Json::Array(Vec::new())),
            ']' => {
                flush_buffer(&mut stack, &mut buffer, &mut current_type, &mut key_stack);
                let item = stack.pop().unwrap();
                last_value = item.clone();
                add_to_previous(&mut stack, key_stack.pop(), item);
            },
            '"' => {
                recording = !recording;
                continue;
            },
            ':' => {
                key_stack.push(buffer.clone());
                buffer.clear();
            },
            ',' => {
                flush_buffer(&mut stack, &mut buffer, &mut current_type, &mut key_stack);
            },
            t => {
                if NUMERIC.contains(&t) && recording == false {
                    buffer.push(t);
                    current_type = 1;
                } else if BOOLCHARS.contains(&t) && recording == false {
                    buffer.push(t);
                    current_type = 2;
                } else if NULLCHARS.contains(&t) && recording == false {
                    buffer.push(t);
                    current_type = 3;
                }
            },
        }
        if recording {
            buffer.push(c);
        }
    }
    last_value
}

fn json_to_python(json: Json, py: Python) -> PyObject {
    match json {
        Json::Null => {
            PyNone.to_py_object(py)
        },
        Json::Bool(value) => {
            PyBool::get(py, value).into_object()
        },
        Json::Number(value) => {
            value.to_py_object(py).into_object()
        },
        Json::String(value) => {
            value.to_py_object(py).into_object()
        },
        Json::Array(arr) => {
            let list = PyList::new(py, &[]);
            for value in arr.into_iter() {
                list.append(py, json_to_python(unbox(value), py));
            }
            list.into_object()
        },
        Json::Object(obj) => {
            let dict = PyDict::new(py);
            for (key, value) in obj.into_iter() {
                dict.set_item(py, key, json_to_python(unbox(value), py));
            }
            dict.into_object()
        },
    }
}

fn json_to_string(json: Json, mut indent: usize, mut comma: bool) -> String {
    let mut result = String::new();
    match json {
        Json::Null => {
            result.push_str("null");
        },
        Json::Bool(value) => {
            result.push_str(&format!("{}", value));
        },
        Json::Number(value) => {
            result.push_str(&format!("{}", value));
        },
        Json::String(value) => {
            result.push_str("'");
            result.push_str(&value);
            result.push_str("'");
        },
        Json::Array(arr) => {
            result.push_str("[");
            result.push_str("\n");
            indent += 4;
            let l = arr.len();
            let mut inner = false;
            for (i, item) in arr.into_iter().enumerate() {
                if i < l - 1 {
                    inner = true;
                } else {
                    inner = false;
                }
                result.push_str(&" ".repeat(indent));
                result.push_str(&json_to_string(unbox(item), indent + 4, inner));
            }
            indent -= 4;
            result.push_str(&" ".repeat(indent));
            result.push_str("]");
        },
        Json::Object(o) => {
            result.push_str("{");
            result.push_str("\n");
            indent += 4;
            let l = o.len();
            let mut inner = false;
            for (i, (key, value)) in o.into_iter().enumerate() {
                //result.push_str(&" ".repeat(indent));
                //result.push_str(&format!("index: {}, len: {}\n", i, l));
                if i < l - 1 {
                    inner = true;
                } else {
                    inner = false;
                }
                result.push_str(&" ".repeat(indent));
                result.push_str("'");
                result.push_str(&key);
                result.push_str("': ");
                result.push_str(&json_to_string(unbox(value), indent, inner));
            }
            indent -= 4;
            result.push_str(&" ".repeat(indent));
            result.push_str("}");
        },
    }
    if comma {
        result.push_str(",");
    }
    result.push_str("\n");
    result
}

fn python_to_json(py: Python, obj: PyObject) -> PyResult<Json> {
    let mut token = match obj.get_type(py).name(py).borrow() {
        "dict" => {
            Json::Object(HashMap::new())
        },
        "list" => {
            Json::Array(Vec::new())
        },
        "str" => {
            Json::String(String::new())
        },
        "bool" => {
            Json::Bool(true)
        },
        "int" => {
            Json::Number(0.0)
        },
        "float" => {
            Json::Number(0.0)
        },
        "NoneType" => {
            Json::Null
        },
        t => panic!("unknown type {}", t),
    };

    match &mut token {
        Json::Null => {
        },
        Json::Bool(value) => {
            *value = obj.extract::<bool>(py)?;
        },
        Json::Number(value) => {
            *value = obj.extract::<f64>(py)?;
        },
        Json::String(value) => {
            *value = obj.extract::<String>(py)?;
        },
        Json::Array(arr) => {
            let list: PyList = obj.cast_into(py)?;
            for value in list.iter(py).into_iter() {
                arr.push(Box::new(python_to_json(py, value)?));
            }
        },
        Json::Object(o) => {
            let dict: PyDict = obj.cast_into(py)?;
            
            for (key, value) in dict.items(py).into_iter() {
                o.insert(key.to_string(), Box::new(python_to_json(py, value)?));
            }
        },
    }
    Ok(token)
}
