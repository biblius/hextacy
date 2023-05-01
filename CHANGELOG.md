# Changelog

## 0.1.3

Change the `=>` in the `adapt!` macro to `as` because it makes more sense.

## 0.1.2

Remove all ACID related traits and drastically simplify how transactions are handled.

Add seaorm to supported drivers.

## 0.1.11-12

Replace r2d2_redis with the redis' crate provided r2d2 implementation

Use cookie through actix instead of directly from the cookie crate
