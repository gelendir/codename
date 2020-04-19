FROM nginx:1.17-alpine

COPY index.html  /var/www/codename/index.html
COPY run.sh /bin/run.sh

ADD https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf /etc/nginx/ssl.conf
ADD https://raw.githubusercontent.com/certbot/certbot/master/certbot/certbot/ssl-dhparams.pem /etc/nginx/ssl-dhparams.pem

CMD ["/bin/run.sh"]
