# rust-http-server
Simple server for downloading files. The server doesn't use any crates, except of regex.

### Usage
You must specify at least the port argument(which goes first):
```
cargo run 8080
```
You can optionally specify an argument for the root directory. By default, the root directory equals the directory you run the command:
```
cargo run 8080 /home/kostya/Downloads
```

The server supports downloading files from the absolute path or from the relative path. 
The server parses the path from the URL query (everything that goes after the port number will be assumed as the path to the file). 

Then it tries to use the path as an absolute path. It checks if the path exists. If does it will send the file in chunks or send an error with code 400, if it founds a directory by this path.
If it fails with an absolute path, it will try the same with a relative path(path will be relative to root directory from parameter, if it exists).
Finally, it will send an error with code 404, if it fails with both tries.

### Other errors
The server is also capable of sending an error 501 if the user tries to use the REST method, which differs from GET or HEAD.
If the server doesn't have enough permission to read the file, it will send an error 401.
And for other errors, it will send an error 500.
