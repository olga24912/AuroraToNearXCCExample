test-counter:
	cd aurora && \
	yarn && \
	cd ../near/contracts && \
	./build.sh && \
	cd ../../aurora/integration-tests && \
	cargo test --all --jobs 4 -- --test-threads 4
