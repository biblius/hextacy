# alx_core

The main crate from which you can import the necessary types to aid you in development. All types here are designed to be as generic as possible, meaning they should not tie implementation details to business logic.

- **crypto**

  Contains various cryptographic helpers for creating secrets, signing data and more.

- **db**

  Contains traits that can be utilised to propagate connections from clients to structs implementing `RepoAccess` traits.

- **ws**

    Module containing a Websocket session handler.

    Every message sent to this handler must have a top level `"domain"` field. Domains are completely arbitrary and are used to tell the ws session which datatype to broadcast.

    Domains are internally mapped to data types. Actors can subscribe via the broker to specific data types they are interested in and WS session actors will in turn publish them whenever they receive any from their respective clients.

    Registered data types are usually enums which are then matched in handlers of the receiving actors. Enums should always be untagged, so as to mitigate unnecessary nestings from the client sockets.
    Uses an implementation of a broker utilising the [actix framework](https://actix.rs/book/actix/sec-2-actor.html), a very cool message based communication system based on the [Actor model](https://en.wikipedia.org/wiki/Actor_model).

    Check out the `web::ws` module for more info and an example of how it works.

- **cache**

  Contains a trait that can be implemented by service caches for easy cache access.
  
  Cache keys are intended to be split into three parts; the domain, the identifier and the actual key.

  All cache accessors must have a `domain` that will serve as the main prefix for their cache keys. The `identifier`
  is used as the second part of that key and determines the subgroup of the domain in which to cache the `key`

  A full cache key would look something like.

  `domain:identifier:key` -> e.g. `auth:login_attempts:user_id`.

  To implement cache identifiers a `CacheIdentifier` trait is available to implement on any enum.

- **logger**
  
  Contains a logger that can be initiated on server start.
