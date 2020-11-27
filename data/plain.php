<?php

if(file_exists("plain.phar")) {
	unlink("plain.phar");
}
$phar = new Phar("plain.phar");
$phar->setStub("<?php __HALT_COMPILER();");
$phar->setMetadata("met");
$phar->addFromString("foo", "bar");
$phar->addFromString("qux", "corge");
$phar->setSignatureAlgorithm(Phar::MD5);
