## About

This is my personal blog.

## Architecture

![img](imges/architecture.webp)

## Status

- [ ] js uses a very old technology (I'm not familiar with the front-end technology)

## Dependences
- Redis
- Postgresql

## Getting Started

### [Rust](https://www.rust-lang.org/)

```
$ curl https://sh.rustup.rs -sSf | sh
```

### [Sqlx Cli](https://github.com/launchbadge/sqlx)
This project use sqlx as Orm framework, so you need to install its command line tool via Rust package manager(eg, Cargo)
```bash
$ cargo install sqlx-cli
```

### [Postgresql](https://www.postgresql.org/)
Use docker images

#### Install from docker-hub
```bash
$ docker pull postgres
$ docker run --name your_container_name -e POSTGRES_PASSWORD=system -d -p 5432:5432 postgres
```

#### If you want to enter psql interactive command line
```bash
$ docker run -it --rm --link your_container_name:postgres postgres psql -h postgres -U postgres
```

#### init database
```bash
$ sqlx database create
$ sqlx migrate run
```

### [Redis](https://github.com/redis/redis)
Use docker images

```bash
$ docker pull redis
$ docker run --name redis -p 6379:6379 -d redis
```

### [Nginx](http://nginx.org/en/download.html)
nginx is only used when deploying production

##### config:
```
server {
        listen       80;
        server_name  127.0.0.1;

        location / {
            proxy_pass http://127.0.0.1:8080;
            proxy_redirect off;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }

        location /api/v1 {
            proxy_pass http://127.0.0.1:8080;
            proxy_redirect off;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }
```

### blog
```
$ cargo run --release // listen on 127.0.0.1:8080
```

if you want to login admin, the account is `admin`, password is `admin`

## Using Docker Compose

This project includes `docker-compose.yml` to run the application together with Postgres and Redis using container-friendly settings.

Quick start (recommended):

1. Copy or create the compose env file. A compose-specific file named `.env.compose` is provided. Do NOT overwrite your development `.env`.

```bash
# if you want to edit values, copy the example first:
cp .env.compose .env.compose.local
# edit .env.compose.local as needed, then
# (optional) move/rename to .env.compose if you prefer
```

2. Build and start services:

```bash
docker compose up --build
# or run in background
docker compose up -d --build
```

3. View logs:

```bash
docker compose logs -f blog
```

4. Database migrations (optional):

If you want the `blog` container to run migrations, either exec into the container and run `sqlx migrate run` (requires `sqlx-cli` available), or run migrations from your host against the compose DB:

```bash
# run migrations from host (requires DATABASE_URL in .env.compose to point to compose db)
sqlx database create
sqlx migrate run
```

Notes
- The project `.env` is reserved for native development and is not used by compose. Compose loads `.env.compose` into the `blog` container.
- `.env.compose` is included in `.gitignore` to avoid committing secrets. Use `.env.example` or `.env.compose` as a template.
- If you change service names or ports in `docker-compose.yml`, update `.env.compose` accordingly.

