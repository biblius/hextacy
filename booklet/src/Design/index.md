# Design

In order to understand why hextacy is built the way it is, we first need to understand how its pieces tie together to provide a flexible project infrastructure.

Hextacy is based on [hexagonal architecture](<https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)>) also known as the _ports and adapters_, _layered_, or _onion_ architecture. You can read great articles about it [here](https://netflixtechblog.com/ready-for-changes-with-hexagonal-architecture-b315ec967749) and [here](https://jeffreypalermo.com/2008/07/the-onion-architecture-part-1/).

At the core of this kind of architectural design is the business layer. The business layer represents the problems the application is designed to solve and as such it largely depends on the requirements. It contains the entity definitions the application will work with. If you take a look at some of the diagrams that are used to represent these architectures, you will always see the business layer in the middle (the core). Each subsequent layer will depend on the previous one and you will usually see arrows pointing from the outer most layers to the inner ones.

_This follows the `D` of SOLID - dependency inversion. For example, at the outer layers of the application is the UI, which depends on the API of various services the application exposes, which depend on the domain entities/services of the business layer. Since the business layer is in the middle, it contains no dependencies and is standalone. As such, the core layer of the application is self-sufficient and should build successfully on its own, even when no concrete implementations are plugged into it._

When we model the application core, we must provide it access to domain entities without coupling it to any concrete way of obtaining those entities. We do so by defining the core logic through behaviour - in rust, we define this behaviour through traits.

As an example, **Repositories** provide methods through which **Adapters** can interact with to get access to application **Entities**.

A repository contains no implementation details about a concrete persistence backend. It is simply an interface which adapters utilise for their specific implementations to obtain the underlying model.

When business level services need access to domain entities, they couple themselves to repositories. By coupling the services only to the repositories, we gain the flexibility of swapping various implementations without ever touching the core logic.

Even though we've talked only about repositories, this paradigm will be present in every aspect of our application. To name a few more examples, our application could contian some caching requirements and some kind of notification mechanism when certain events occur. We also want to design those in a manner where we hide away the implementations of those mechanisms from the core logic.

In addition to having the _internals_ decoupled, we are also decoupled from any potential _interactors_ the application could use to access the core. You can think of interactors as the front-end to the application - since we have a standalone core, it becomes irrelevant whether we use HTTP, a desktop program or a CLI to access it.

Keeping all of this in mind, we will next explore how we can implement these patterns in rust.
