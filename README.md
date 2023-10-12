# Overview

This is a toy HTTP server that implements:

- `GET /`: Return 200 Ok
- `GET /echo/data`: Return a 200 Ok response with the `data` as its body
- `GET /user-agent`: Return a 200 Ok response wih the request's `User-Agent` header value
- `GET /files/test.txt`: Return whether `test.txt` exists in the dir of `--directory <dir>`
- `POST /files/test.txt -d @test.txt`: Accept the request body and save it to `--directory <dir>`

The purpose of this repository is to showcase the various capabilities of the Rust programming language.