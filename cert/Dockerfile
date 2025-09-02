FROM alpine:latest
WORKDIR /app

RUN apk add --no-cache openssl
COPY ./make-tls-certs.sh /app/make-tls-certs.sh

ENTRYPOINT ["sh", "/app/make-tls-certs.sh"]