# Rust Web Tools

This repo is deisgned to quick start web server development with [actix_web](https://actix.rs/) by providing out of the box basic user authentication flows, session middleware, a simple CLI tool for file managment and a bunch of utilities for the web.

This kind of project structure is heavily based on [hexagonal architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)) also known as the *ports and adapters* architecture which is very flexible and easily testable. You can read great articles about it [here](https://netflixtechblog.com/ready-for-changes-with-hexagonal-architecture-b315ec967749) and [here](https://blog.phuaxueyong.com/post/2020-05-25-what-architecture-is-netflix-using/).

The heart of this repository are the infrastructure and server directories, so let's start from there.

## **Infrastructure**

Contains the foundations from which we build our servers. Here you'll find all the database clients, adapters and repositories as well as a bunch of crypto, web and actor helpers.

The most interesting here is the store module, where the said data sources are located. It is divided in to three parts:

### **Store**

#### **Repository**

Contains data structures and the interfaces with which we interact with them. Their sole purpose is to describe the nature of interaction with the database, they are completely oblivious to the implementation. This module is designed to be as generic as possible and usable in any server domain module.

#### **Adapters**

This module contains the client specific implementations of the repository interfaces. Adapters adapt the behaviour dictated by their underlying repository. Seperating implementation from behaviour decouples any other module using a repository from the client specific code located in the adapter.

#### **Models**

This is where application models are located. These aren't necessarily meant to be stored in databases and serve as utility structures for responses, the cache and intermediary data that can be used across the project.

The store adapters utilize connections established from the clients module:

### **Clients**

Contains structures implementing client specific behaviour such as connecting to and establishing connection pools with database, cache, smtp and http servers. All the connections made here are generally shared throughout the app with Arcs.

### **Actors**

Module containing an implementation of a basic broadcastable message and a broker utilising the [actix framework](https://actix.rs/book/actix/sec-2-actor.html), a very cool message based communication system based on the [Actor model](https://en.wikipedia.org/wiki/Actor_model).

The rest is a bunch of helpers that don't require that much explanation.

## **Server**

The main application binary. Contains the domain logic and the request handlers.
