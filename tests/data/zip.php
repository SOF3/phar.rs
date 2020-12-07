<?php

if(file_exists("zip.phar")) {
	unlink("zip.phar");
}
$phar = new Phar("zip.phar");
$phar->compressFiles(Phar::GZ);
$phar->setStub("<?php __HALT_COMPILER();");
$phar->setMetadata("met");
$phar->addFromString("foo", "bar");
$phar->addFromString("qux", "corge");
$phar->setSignatureAlgorithm(Phar::MD5);
