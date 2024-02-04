all: frontend backend
	cp target/x86_64-unknown-linux-musl/release/backend build/blog

debug: frontend_debug backend_debug
	cp target/x86_64-unknown-linux-musl/debug/backend build/blog

frontend_debug:
	cd frontend && \
	wasm-pack build --target web --out-name blog --out-dir ../build && \
	cd ..  && \
	cp build/blog_bg.wasm asset/
	cp build/blog.js asset/

backend_debug:
	cargo +nightly build -p backend --target x86_64-unknown-linux-musl

frontend:
	cd frontend && \
	wasm-pack build --release --target web --out-name blog --out-dir ../build && \
	cd ..  && \
	cp build/blog_bg.wasm asset/
	cp build/blog.js asset/

backend:
	cargo +nightly build -p backend --release --target x86_64-unknown-linux-musl

clean:
	rm -rf build || true
	rm asset/blog_bg.wasm || true
	cargo clean || true

.PHONY: frontend backend clean all debug frontend_debug backend_debug
