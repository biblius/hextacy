# **⬡ Hextacy ⬡**

A repository designed to bootstrap backend development.

You can read more about hextacy in [the booklet](https://biblius.github.io/hextacy/).

Hextacy is a work in progress:

- [x] Database drivers (SQL(diesel, seaorm), Mongo)
- [x] Cache drivers (Redis, TODO: Memcachd)
- [x] Notifications (Email via SMTP)
- [x] Message Queue (Amqp, Redis)
- [ ] Scheduled jobs (crons with tokio-cron)
- [ ] CLI tool for creating app infrastructure (in progress)
- [ ] Something probably

## **Feature flags**

```bash
  # Enable everything, sql default is postgres
  - full

  # Enable http, cookie and mime crates
  - web

  # Enable lettre and a simple template SMTP mailer
  - email

  # Enable the specified backend for the specified driver
  - db - postgres|mysql|sqlite - diesel|seaorm
  - db-mongo

  # Enable the redis driver and an in memory cache for quickly prototyping
  - cache-redis
  - cache-inmem
```
