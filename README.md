# serde-arma

Use Arma configs as Rust structs or convert to JSON, TOML, or any serde compatible format.

# Example

```
string = "data";
class MoreData {
    numbers[] = {1,2,3};
    int = 14;
}
```
*serde magic*
```json
{
    "string": "data",
    "MoreData": {
        "numbers": [1,2,3],
        "int": 14
    }
}
```
