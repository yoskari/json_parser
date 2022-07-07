import json_parser

j = r'''{
    "list": [1, 2, 3], 
    "bool": true, 
    "false": false,
    "float": 1.11, 
    "number": -123, 
    "unicode": "Слава Украине",
    "null": null,
    "dict": {
        "foo": "bar", 
        "list": [
            3, 2, 1
        ]
    }
}'''
print("INPUT DATA:")
print(j)

print("\n\nOUTPUT")
print(json_parser.loads(j))
