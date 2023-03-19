# OPEN SSL SETUP

The `openssl.sh` script in this directory can be utilised to quickly generate a CA and a self signed certificate to enable HTTPS for development on local connections.

Make sure to `cd` in the script's directory and then execute it so your files get generated neatly in an isolated directory. All the files generated are git ignored.

## Setup

Run the script to generate the CA private key (1), the CA (2), the certificate private key (3), a temporary certificate signing request (4), a temporary config file for the certificate (5) and finally the certificate (6).

- 1. Generate the private key for the CA certificate. This key is private and should NEVER be accessible. You can additionally encrypt it with a passphrase by adding `-aes256`.

      ```bash
      openssl genrsa -out 'ca-key.pem' 4096
      ```

- 2. Generate a CA certificate with the private key from the above step. You can change the duration and file names if you want to, just make sure these changes reflect on the rest of the script. The `subj` is used to identify the CA in the browser. You will be prompted for a password if you encrypted the private key from the previous step.

      ```bash
      openssl req -new -x509 -sha256 -days 365 -key 'ca-key.pem' -out 'ca.pem' -subj "/C=HR/ST=OS/L=Osijek/O=Alchemy/OU=Clandestine/CN=localhost"
      ```

      To read the file in human readable format, use the following command:

      ```bash
      openssl x509 -in 'ca.pem' -text
      ```

- 3. Generate a private key for the certificate,

      ```bash
      openssl genrsa -out 'key.pem' 4096
      ```

- 4. Generate a temporary certificate signing request.

      ```bash
      openssl req -new -sha256 -subj "/C=HR/ST=OS/L=Osijek/O=Alchemy/OU=Clandestine/CN=localhost" -key 'key.pem' -out 'cert.csr'
      ```

- 5. Create a temporary config file for the certificate. Contains DNS names or IP addresses that the cert will be valid for. You can add more if you want.

      ```bash
      echo "subjectAltName=DNS:localhost,IP:127.0.0.1" >extfile.cnf
      ```

- 6. Finally, create the certificate using the generated config file and append a serial number to it with -CAcreateserial.

      ```bash
      openssl x509 -req -sha256 -days 365 -in 'cert.csr' -CA 'ca.pem' -CAkey 'ca-key.pem' -out 'cert.pem' -extfile 'extfile.cnf' -CAcreateserial
      ```

There should be 4 files in total. After they've been successfully generated, depending on your machine you will have to add the `ca.pem` to your machine's trusted CAs since it was used to sign the actual certificate.

- Mac - Open up the *Keychain Acces* app, head to the *System* section and drag and drop the `ca.pem` file to it. Double click the file, expand the *Trust* section and set 'When using this certificate' to *Always trust*.

- Ubuntu - Rename the `ca.pem` to `ca.crt` and place it in `usr/local/share/ca-certificates`, then run the command `sudo update-ca-certificates`. Read more [here](https://superuser.com/questions/1430089/how-to-add-a-self-signed-ssl-certificate-to-linux-ubuntu-alpine-trust-store) and [here](https://superuser.com/questions/437330/how-do-you-add-a-certificate-authority-ca-to-ubuntu).

Once you're all set with the keys, make sure actix is imported with the `openssl` feature flag:

```toml
actix-web = { version = "4", features = ["openssl"] }
openssl = { version = "0.10", features = ["v110"] }
```

The certificate is valid only for localhost, so we'll bind to `127.0.0.1` using the `bind_openssl` function. When setting up the server we first have to build the ssl acceptor:

```rust
let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
builder
    .set_private_key_file("./openssl/key.pem", SslFiletype::PEM)
    .unwrap();
builder
    .set_certificate_chain_file("./openssl/cert.pem")
    .unwrap();
```

Then bind like:

```rust
HttpServer::new(move || {
    App::new()
        /*
        ...
        */
})
.bind_openssl("127.0.0.1:8000", builder)?
.run()
```

The server should now be securely accessible on `https://localhost:8000`
