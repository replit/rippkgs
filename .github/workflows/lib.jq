def assert(cond; $message):
	if cond then null else error($message) end
	;

def assertField($input; $field; $expect):
	assert($input | has($field); "expected field `" + $field + "` in result")
	| $input | .[$field] | . as $actual
	| assert(
			$actual == $expect;
			"field `" + $field + "` has unexpected value"
			+ "\n  got:      " + ($actual | tostring)
			+ "\n  expected: " + ($expect | tostring)
		)
	;
