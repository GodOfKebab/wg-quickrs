# If /app/certs/root does NOT exist, create it
if [ ! -d /app/certs/root ]
then
    mkdir /app/certs/root
fi

# Create a Root Certificate and self-sign it
# If /app/certs/rootCA.key does NOT exist, create it
if [ ! -f /app/certs/root/rootCA.key ]
then
    # Create the root key
    openssl genrsa -out /app/certs/root/rootCA.key 4096
fi

# If /app/certs/rootCA.crt does NOT exist, create it
if [ ! -f /app/certs/root/rootCA.crt ]
then
    # Generate the Root Certificate.
    openssl req -x509 -sha256 -new -noenc -key /app/certs/root/rootCA.key -days 800 -out /app/certs/root/rootCA.crt  -subj "/C=$COUNTRY/ST=$STATE/L=$LOCALITY/O=$ORGANIZATION CA/OU=$ORGANIZATIONAL_UNIT/CN=$ROOT_CN"
fi

# If /app/certs/servers does NOT exist, create it
if [ ! -d /app/certs/servers ]
then
    mkdir /app/certs/servers
fi

# Create certificates for servers
for server in "$@"
do
    # If /app/certs/servers/$server does NOT exist, create it
    if [ ! -d "/app/certs/servers/$server" ]
    then
        mkdir "/app/certs/servers/$server"
    fi

    # Create the certificate's key if it doesn't exist
    if [ ! -f "/app/certs/servers/$server/key.pem" ]
    then
        openssl genpkey -algorithm RSA \
            -out "/app/certs/servers/$server/key.pem" \
            -pkeyopt rsa_keygen_bits:4096
    fi

    # Generate the Certificate Signing Request (CSR)
    openssl req -new \
        -key "/app/certs/servers/$server/key.pem" \
        -out "/app/certs/servers/$server.csr" \
        -subj "/C=$COUNTRY/ST=$STATE/L=$LOCALITY/O=$ORGANIZATION certificate/OU=$ORGANIZATIONAL_UNIT/CN=$server"

    # Configure extensions so browsers don't yell
    cat > "/app/certs/servers/$server.ext" << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = $server
EOF

    # Finally create the certificate for the server and sign it using CA
    openssl x509 -req \
        -in "/app/certs/servers/$server.csr" \
        -CA /app/certs/root/rootCA.crt \
        -CAkey /app/certs/root/rootCA.key \
        -CAcreateserial \
        -out "/app/certs/servers/$server/cert.pem" \
        -days 825 -sha256 \
        -extfile "/app/certs/servers/$server.ext"

    # Remove unnecessary files: Certificate Signing Request (CSR).
    rm "/app/certs/servers/$server.csr" "/app/certs/servers/$server.ext"

    # Verify the newly created certificate
    echo "Created a certificate and a key for '$server' at /app/certs/servers/$server/cert.pem and /app/certs/servers/$server/key.pem"
    openssl x509 -in "/app/certs/servers/$server/cert.pem" -text -noout
done

