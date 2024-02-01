def call(routes, **kwargs):
    direct_responses = [
        {
            "match": {
                "path": instance["path"]
            },
            "direct_response": {
                "status": 200,
                "body": {
                    "inline_string": instance["body"]
                }
            }
        }
        for instance in routes
    ]
    
    yield {
        "name": "rds_routes",
        "virtual_hosts": [
            {
                "name": "local_service",
                "domains": [
                    "*"
                ],
                "routes": [
                    *direct_responses,
                    {
                        "match": {
                            "prefix": "/"
                        },
                        "route": {
                            "cluster": "httpbin_cluster"
                        }
                    }
                ]
            }
        ]
    }
