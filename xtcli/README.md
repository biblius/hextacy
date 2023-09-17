# XTC - Very much a work in progress

A.K.A. the CLI tool provides a way of seamlessly generating endpoints and middleware.

To set up the cli tool after cloning the repository enter

```bash
cargo install --path xtc
```

from the project root.

The list of top level commands can be viewed with the xtc -h command.

The most notable commands are `[g]enerate` which sets up endpoint/middleware boilerplate and `[anal]yze` which scans the router and middleware directories and constructs a Json/Yaml file containing endpoint info.

Xtc only works for the project structure described in the architecture section.

The `[g]enerate` command generates an endpoint structure like the one described in the router. It can generate route `[r]` and `middleware [mw]` boilerplate. Apis can also supplied to the command with the `-c` flag followed by the apis you wish to hook up to the endpoint, comma seperated e.g.

```bash
xtc gen route <NAME> -c repository,cache
```

This will automagically hook up the apis to the service and set up an infrastructure boilerplate. It will also append `pub(crate) mod <NAME>` to the router's mod.rs. It also takes in a `-p` argument which can be used to specify the directory you want to set up the endpoint.
