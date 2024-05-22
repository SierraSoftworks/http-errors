# `http-errors`
**Lightweight HTTP errors server designed for use with Traefik.**

This server is designed to be used with Traefik as a fallback server for HTTP errors.
It is written in Rust, designed to be ultra-lightweight and fast, and presents a
minimal security footprint to minimize the risk of exploitation.

By default it'll serve a configurable error page for any HTTP request received, however you can also serve specific pages by visiting `/.well-known/http-${status-code}` instead.

To configure the default status code, you can set the `DEFAULT_STATUS_CODE` environment variable. By default it is set to `501` (which returns a `Not Implemented` error).
