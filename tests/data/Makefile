.PHONY: all
all: plain.phar zip.phar ssl.phar

plain.phar: plain.php; php -dphar.readonly=0 plain.php

zip.phar: zip.php; php -dphar.readonly=0 zip.php

ssl.phar: ssl.php ssl-private.pem ssl.phar.pubkey
	php -dphar.readonly=0 ssl.php

ssl-private.pem: Makefile
	openssl genrsa -out ssl-private.pem 4096

ssl.phar.pubkey: Makefile ssl-private.pem
	openssl rsa -in ssl-private.pem -pubout -out ssl.phar.pubkey
