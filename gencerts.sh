#!/bin/bash
mkdir -p data/ssl
openssl req -x509 -newkey rsa:4096 -keyout data/ssl/certificate.key -out data/ssl/certificate.crt -days 365 -subj '/CN=codename.localhost' -nodes
