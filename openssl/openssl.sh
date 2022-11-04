# Run this script to generate the CA private key (1), the CA (2),
# the certificate private key (3), the certificate signing request (4), the config file for the certificate (5),
# the actual certificate (6).

### MAKE SURE TO CD IN TO THIS SCRIPT'S DIRECTORY BEFORE YOU RUN IT

# (1) Generate private key for the CA certificate. To encrypt this with a password add -aes256
openssl genrsa -out 'ca-key.pem' 4096

# (2) Generate the CA.
openssl req -new -x509 -sha256 -days 365 -key 'ca-key.pem' -out 'ca.pem' -subj "/C=HR/ST=OS/L=Osijek/O=Myco/OU=Myco/CN=localhost"

# (3) Generate a private key for the certificate.
openssl genrsa -out 'key.pem' 4096

# (4) Generate a certificate signing request.
openssl req -new -sha256 -subj "/C=HR/ST=OS/L=Osijek/O=Myco/OU=Myco/CN=localhost" -key 'key.pem' -out 'cert.csr'

# (5) Create a config file for the certificate. Contains DNS names or IP addresses that the cert will be valid for.
echo "subjectAltName=DNS:localhost,IP:127.0.0.1" >extfile.cnf

# (6) Finally, create the certificate using the generated config file and append
# a serial number to it with -CAcreateserial.
openssl x509 -req -sha256 -days 365 -in 'cert.csr' -CA 'ca.pem' -CAkey 'ca-key.pem' -out 'cert.pem' -extfile 'extfile.cnf' -CAcreateserial

# Remove temp files
rm extfile.cnf
rm ca.srl
rm cert.csr
