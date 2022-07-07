import json_parser

j = {
        'list': [1, 2, 3], 
        'bool': True, 
        'float': 1.11, 
        'dict': {
            'foo': 'bar', 
            'list': [
                3, 2, 1
            ]
        }
    }

print(json_parser.dumps(j))
