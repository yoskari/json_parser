# json parser for python written using rust
This was a simple test project and the parsing is done without a proper lexer
using a stack method, which means there are some limitations to what it can parse properly.

## Json structure
```rust
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Box<Json>>),
    Object(HashMap<String, Box<Json>>)
}
```

## installation
```bash
cargo build --release
cp target/release/libjson_parser.so json_parser.so
```

## use in python
```python
import json_parser

# convert a Python dictionary into a json-formatted string
example = {"foo": True, "bar": None, "list": [1, 2, 3]}
print(json_parser.dumps(example))

# convert a json-formatted string into a python object
json = '''{
    "list": [1, 2, 3], 
    "bool": true, 
    "false": false,
    "float": 1.11, 
    "number": -123, 
    "unicode": "Слава Украине",
    "null": null,
    "dict": {
        "foo": "bar\041\u2b78\n", 
        "list": [
            3, 2, 1
        ]
    }
}'''
example = json_parser.loads(json)
print(exampe)

```
