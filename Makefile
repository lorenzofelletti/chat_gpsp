.PHONY: code-after-macros
code-after-macros:
	cargo rustc --profile=check -- -Zunpretty=expanded | less

.PHONY: build
build:
	cargo psp --release
