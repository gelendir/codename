#!/bin/ash
(
    while true; do
        sleep 6h
        nginx -s reload
    done
) &
nginx -g "daemon off;"
