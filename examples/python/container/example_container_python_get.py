
def manifest():
    return {
        "type_name": "example_container_python_get",
        "container_type_name": "example_container_python",
        "arguments" : {
        }, 
    }

def example_container_python_get(container):
    return container.value