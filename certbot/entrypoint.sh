#!/bin/ash

if [ "$1" = "codename" ]; then
    while true; do
        certbot renew
        sleep 12h
    done
else
    exec $@
fi
