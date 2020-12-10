<?php

$phar = new Phar($argv[1]);

function assertEquals($a, $b, string $err) {
	if($a !== $b) throw new AssertionError(sprintf("%s: %s !== %s", $err, json_encode($a), json_encode($b)));
}

assertEquals($phar->getStub(), "<?php __HALT_COMPILER(); ?>\r\n", "stub mismatch");
assertEquals($phar->getMetadata(), null, "metadata mismatch");
assertEquals(file_get_contents($phar["foo"]), "bar", "metadata mismatch");
assertEquals(file_get_contents($phar["qux"]), "corge", "metadata mismatch");
