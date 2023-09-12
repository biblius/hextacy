# Server example with contracts

This example is set up with services that use contracts. In this example, the repository traits do not
take in self. Services specify contracts to use for grouping together actions on data sources.
Contracts are created via component impl blocks, they are traits which have the same signatures as the
functions of an impl block and are used to hide away the implementation details of the component.

## **Setup**

Read more about the openssl setup in `openssl/README.md`.

1. Create a real `.env` file filling out the example file with your desired parameters and create the database you entered in the file. You do not have to fill out fields you don't intend to use.

    - For the Email part, to use an existing gmail, use [this spec](https://support.google.com/mail/answer/7126229?hl=en#zippy=%2Cstep-change-smtp-other-settings-in-your-email-client) to set up the SMTP host(smtp.gmail.com) and port(465) and [follow these instructions](https://support.google.com/accounts/answer/185833?hl=en#zippy=%2Cwhy-you-may-need-an-app-password) to generate an app password. The password can then be used for the `SMTP_PASSWORD` variable. For the sender and username enter your email address.

2. `cd` into the openssl directory and run `./openssl.sh`, follow the instructions from that directory to set up the certificates.

3. Import the postman collection from the `resources` to postman and run with

```bash
cargo run --example server
```

## **Authentication flow**

The user is expected to enter their email and password after which an email with a registration token gets sent (`start_registration`).

Users can request another token if their token expires (`resend_registration_token`).

Once the user verifies their registration token they must log in, after which they will receive a session ID cookie and a CSRF token in the header (`verify_registration_token`, `login`).

The cookie and token are then used by the middleware to authenticate the user when accessing protected resources. It does so by grabbing both from the request and trying to fetch a session first from the cache, then if that fails from postgres. The session is searched for by ID and must be unexpired and have a matching csrf token, otherwise the middleware will error.

A route exists for setting a user's OTP secret and a session must be established to access it (`set_otp_secret`).

When a user sets their OTP secret they have to provide a valid OTP after successfully verifying credentials or they won't be able to establish a session (`verify_otp`).

Users can change their password and logout only if they have an established session. On logout a user can also choose to purge all of their sessions (`change_password`, `logout`).

If a user changes their password their sessions will be purged and they will receive an email notifying them of the change with a password reset token in case it wasn't them. The PW reset token lasts for 2 days (`reset_password`).

Users who forgot their passwords can request a password reset. They will receive an email with a temporary token they must send upon changing their password for the server to accept the change. Once they successfully change it their sessions will be purged and a new one will be established (`forgot_password`, `verify_forgot_password`).

### **A note on middleware**
  
  The structure is similar to the endpoints as demonstrated above. If you're interested in a bit more detail about how Actix's middleware works, [here's a nice blog post you can read](https://imfeld.dev/writing/actix-web-middleware). By wrapping resources with middleware we get access to the request before it actually hits the handler. This enables us to append any data to the request for use by the designated handler. Essentially, we have to implement the `Transform` trait for the middleware and the `Service` trait for the actual business logic.

  If you take a look at the `auth` middleware you'll notice how our `Transform` implementation, specifically the `new_transform` function returns a future whose output value is a result containing either the `AuthMiddleware` or an `InitError` which is a unit type. If you take a look at the signature for Actix's `wrap` function you can see that we can pass to it anything that implements `Transform`. This means that, for example, when we want to wrap a resource with our `AuthGuardMiddleware`, we have to pass the instantiated `AuthGuard` struct, because that's the one implementing `Transform`.
  If you take an even closer look at what happens in `wrap` you'll see that it triggers `new_transform` internally, meaning the instantiated `AuthGuard` transforms into an `AuthGuardMiddleware` which executes all the business.

  The structure is exactly the same as that of endpoints with the exception of **interceptor.rs** which contains our `Transform` and `Service` implementations. The main functionality of the middleware is located in the `call` function of the `Service` implementation.

The helpers module contains various helper functions usable throughout the server.

### **Storage Directory Overview**

The storage crate is project specific which is why it's completely seperated from the rest. It contains 3 main modules. Normally, there isn't a need for diesel AND seaorm postgres adapters, this example is just showcasing how they can be used interchangeabley

- **Repository**

    Contains interfaces for interacting with application models. Their sole purpose is to describe the nature of interaction with the database, they are completely oblivious to the implementation. This module is designed to be as generic as possible and usable anywhere in the service logic.

- **Adapters**

    Contains the driver specific implementations of the repository interfaces. Adapters adapt the behaviour dictated by their underlying repository. Seperating implementation from behaviour decouples any other module using a repository from the driver specific code located in the adapter.

- **Models**

    Where application models are located.

The storage adapters can utilize connections established from the drivers module.
