# Etapa 1: Imagem base
FROM rust:latest as builder

# Instalar dependências do sistema
RUN apt-get update && apt-get install -y libssl-dev

# Definir diretório de trabalho
WORKDIR /usr/src/app

# Copiar arquivos do projeto
COPY . .

# Baixar as dependências do Cargo
RUN cargo build --release

# Etapa 2: Imagem final
FROM debian:bullseye-slim

# Instalar dependências necessárias para rodar o app
RUN apt-get update && apt-get install -y libssl1.1

# Copiar o binário compilado da etapa anterior
COPY --from=builder /usr/src/app/target/release/app /usr/local/bin/

# Definir o comando de execução
CMD ["app"]
