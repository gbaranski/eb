# Protocol

JSON-RPC is used for communication between frontend and backend.

## API methods

### `insert`

Inserts a string at given position.

Request:
```json
{
    "content": "abc"
}
```


### `open`

Opens a resource, usually a file.

Request:
```json
{
    "url": "file:///project-a/main.rs"
}
```


