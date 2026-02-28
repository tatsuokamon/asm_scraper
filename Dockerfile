# backend build
	FROM rust:1.92 AS backend-builder
	WORKDIR /app

	COPY ./Cargo.toml ./
	COPY ./src ./src

	RUN cargo build --release

# migration build
	FROM rust:1.92 AS migration-builder
	WORKDIR /app

	COPY ./migration/Cargo.toml ./
	COPY ./migration/src/ ./src/

	RUN cargo build --release
	RUN ls ./target

# running
	FROM gcr.io/distroless/cc-debian12
	WORKDIR /app

	COPY --from=backend-builder /app/target/release/backend .
	COPY --from=migration-builder /app/target/release/migration .
	COPY ./.env .

	ENV PORT=8080
	EXPOSE 8080

	# need set environment var: DATABASE_URL
	# CMD ["./migration", "up"]
	# CMD ["./app"]
