# Étape 1 : Build de l'application Rust
FROM rust:1.75-slim as builder

WORKDIR /app

# Installer les dépendances système nécessaires
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copier les fichiers de configuration et de dépendances
COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY services/Cargo.toml ./services/
COPY models/Cargo.toml ./models/
COPY repositories/Cargo.toml ./repositories/
# Astuce pour le cache : créer des dossiers vides
RUN mkdir api/src services/src models/src repositories/src
RUN echo "fn main() {}" > api/src/main.rs
RUN echo "fn main() {}" > services/src/main.rs
RUN echo "fn main() {}" > models/src/main.rs
RUN echo "fn main() {}" > repositories/src/main.rs
RUN cargo build --release || true

# Copier le code source réel
COPY . .

# Build final
RUN cargo build --release

# Étape 2 : Image minimale pour l'exécution
FROM debian:bookworm-slim

WORKDIR /app

# Installer les librairies nécessaires à l'exécution
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copier le binaire compilé et le fichier .env
COPY --from=builder /app/target/release/swaptun-backend /app/
COPY .env /app/

# Créer un utilisateur non-root
RUN useradd -m appuser
USER appuser

EXPOSE 8000

CMD ["/app/swaptun-backend"]
