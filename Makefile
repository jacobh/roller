.PHONY: check

check:
	cargo check && (cd web_ui && make check)
