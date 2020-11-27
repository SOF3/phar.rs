<?php

if(file_exists("ssl.phar")) {
	unlink("ssl.phar");
}

$sk = file_get_contents("ssl-private.pem");

$phar = new Phar("ssl.phar");
$phar->setStub("<?php __HALT_COMPILER();");
$phar->setMetadata("met");
$phar->addFromString("foo", "bar");
$phar->addFromString("qux", "corge");
$phar->setSignatureAlgorithm(Phar::OPENSSL, $sk);
