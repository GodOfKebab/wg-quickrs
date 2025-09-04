#!/usr/bin/env bash
set -euo pipefail

# Detect OpenSSL flavor and set correct flag
OPENSSL_REQ_NOENC_FLAG="-noenc"
if openssl version 2>/dev/null | grep -qi "LibreSSL"; then
    # macOS (LibreSSL doesn't support -noenc)
    OPENSSL_REQ_NOENC_FLAG="-nodes"
fi

# If certs/root does NOT exist, create it
if [ ! -d certs/root ]
then
    mkdir -p certs/root
fi

# Create a Root Certificate and self-sign it
# If certs/rootCA.key does NOT exist, create it
if [ ! -f certs/root/rootCA.key ]
then
    # Create the root key
    openssl genrsa -out certs/root/rootCA.key 4096
fi

# If certs/rootCA.crt does NOT exist, create it
if [ ! -f certs/root/rootCA.crt ]
then
    # Generate the Root Certificate.
    openssl req -x509 -sha256 -new $OPENSSL_REQ_NOENC_FLAG \
        -key certs/root/rootCA.key \
        -out certs/root/rootCA.crt \
        -subj "/C=$COUNTRY/ST=$STATE/L=$LOCALITY/O=$ORGANIZATION CA/OU=$ORGANIZATIONAL_UNIT/CN=$ROOT_CN" \
        -days 800
fi

# If certs/servers does NOT exist, create it
if [ ! -d certs/servers ]
then
    mkdir -p certs/servers
fi

# Create certificates for servers
for server in "$@"
do
    # If certs/servers/$server does NOT exist, create it
    if [ ! -d "certs/servers/$server" ]
    then
        mkdir "certs/servers/$server"
    fi

    # Create the certificate's key if it doesn't exist
    if [ ! -f "certs/servers/$server/key.pem" ]
    then
        openssl genpkey -algorithm RSA \
            -out "certs/servers/$server/key.pem" \
            -pkeyopt rsa_keygen_bits:4096
    fi

    # Generate the Certificate Signing Request (CSR)
    openssl req -new \
        -key "certs/servers/$server/key.pem" \
        -out "certs/servers/$server.csr" \
        -subj "/C=$COUNTRY/ST=$STATE/L=$LOCALITY/O=$ORGANIZATION certificate/OU=$ORGANIZATIONAL_UNIT/CN=$server"

    # Configure extensions so browsers don't yell
    cat > "certs/servers/$server.ext" << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = $server
EOF

    # Finally create the certificate for the server and sign it using CA
    openssl x509 -req \
        -in "certs/servers/$server.csr" \
        -CA certs/root/rootCA.crt \
        -CAkey certs/root/rootCA.key \
        -CAcreateserial \
        -out "certs/servers/$server/cert.pem" \
        -days 825 -sha256 \
        -extfile "certs/servers/$server.ext"

    # Remove unnecessary files: Certificate Signing Request (CSR).
    rm "certs/servers/$server.csr" "certs/servers/$server.ext" certs/root/rootCA.srl

    # Verify the newly created certificate
    echo "Created a certificate and a key for '$server' at certs/servers/$server/cert.pem and certs/servers/$server/key.pem"
    openssl x509 -in "certs/servers/$server/cert.pem" -text -noout
done

