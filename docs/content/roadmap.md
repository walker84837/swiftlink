# Roadmap

This document outlines planned improvements and long-term goals.

## Functionality

- Optional authentication for link creation set in the server's config
- Complete client SDK support
- Improve error handling

## Deployment

- Official Docker image
- docker-compose examples

This roadmap is subject to change.

## Roadmap

### Functionality

* [ ] Implement authentication support more broadly (e.g., for `POST /api/create`).
* [ ] Address the type duplication in `swiftlink-server` by consistently using types from `swiftlink-api`.
* [ ] Complete the `async` client in `swiftlink-api` by adding the `delete_link` method.
* [ ] Improve error handling and provide more descriptive error responses.
* Landing page:
  * [ ] Simple install with `curl` command and `install.sh` file.
* [ ] Rate limiting
* [ ] CORS headers

### Look and feel

* Landing page
  * [ ] Make project logo for `favicon.ico`.
  * [ ] Develop a templated and "specialized" page for the `swiftlink-server` with dynamic content (e.g., domain, port) and information on the service that's running.
