# Some note

Below some note about crash course on http

## Basic stuff

There is `2** = 65536` ports:

- http-prod: 80
- https-prod: 433
- http-dev: 8080
- https-dev: 8433

```bash
printf 'HEAD / HTTP/1.1\r\nHost: neverssl.com\r\nConnection: close\r\n\r\n' | nc neverssl.com 

HEAD / HTTP/1.1
Host: neverssl.com
Connection: close
```

Every line ends with `\r\n`, also known as CRLF, for Carriage Return + Line Feed, that's right,
HTTP is based on teletypes, which are just remote typewriters

```text
dig +short A neverssl.com
dig +short AAAA neverssl.com
```

Error code:

- 404 -> Not fonund
- 403 -> Forbidden
- 200 -> Ok
- 301 -> Permanently moved
- 500 -> Internal server Error

HTTP verb:

- HEAD
- GET
- POST
- DELETE
- OPTIONS => CORS
