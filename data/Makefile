plain.phar: plain.php; php plain.php

zip.phar: zip.php; php zip.php

ssl.phar: ssl.php ssl-private.pem ssl.phar.pubkey
	php ssl.php

ssl-private.pem: Makefile
	openssl genrsa -out ssl-private.pem 4096

ssl.phar.pubkey: Makefile ssl-private.pem
	openssl rsa -in ssl-private.pem -pubout -out ssl.phar.pubkey
