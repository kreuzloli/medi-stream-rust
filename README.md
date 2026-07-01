# medi-stream-rust

Axum rewrite of the original Spring Boot `medi-stream` service.

## Run

```bash
cp .env.example .env
# fill DATABASE_URL, REDIS_URL, JWT_SECRET_BASE64
cargo run
```

The service exposes the same main route groups:

- `POST /auth/login`
- `GET /auth/me`
- `GET /catalog/departments?includeDiseases=false`
- `GET /catalog/departments/{deptId}/diseases`
- `GET /catalog/full`
- `POST /account`
- `GET /account/{id}`
- `PUT /account/{id}`
- `DELETE /account/{id}`
- `GET /account?page=1&size=10&userCode=xxx`

`/account` routes require `Authorization: Bearer <token>`. The current login behavior matches the Java demo: `admin / 123456`.
