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

### Architecture

- **Network Isolation**: PostgreSQL and Redis are only accessible within the internal network (not exposed to the host). Only the `blog` service exposes port 8080 to the outside.
- **Automatic Database Migration**: The `db-migrate` service automatically runs `sqlx migrate run` on startup to initialize the database schema.
- **Health Checks**: Services include health checks to ensure proper startup order.

### Quick Start

1. Copy or create the compose env file. A compose-specific file named `.env.compose` is provided. Do NOT overwrite your development `.env`.

```bash
# If you want to edit values, copy the example first:
cp .env.compose .env.compose.local
# Edit .env.compose.local as needed, then rename it
mv .env.compose.local .env.compose
```

2. Build and start services:

```bash
docker compose up --build
# Or run in background
docker compose up -d --build
```

3. View logs:

```bash
docker compose logs -f blog
```

### Notes

- Database migrations run automatically via the `db-migrate` service. No manual migration is required.
- PostgreSQL and Redis ports are NOT exposed to the host for security. They are only accessible within the Docker internal network.
- The `blog` service will wait for:
  - PostgreSQL to be healthy
  - Redis to be healthy
  - Database migrations to complete successfully
- The project `.env` is reserved for native development and is not used by compose. Compose loads `.env.compose` into the `blog` container.
- `.env.compose` is included in `.gitignore` to avoid committing secrets. Use `.env.example` or `.env.compose` as a template.
- If you change service names in `docker-compose.yml`, update `.env.compose` accordingly.

