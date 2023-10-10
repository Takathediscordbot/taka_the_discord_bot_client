FROM debian:bullseye-slim
WORKDIR /taka_the_discord_bot_client

RUN apt-get update
RUN apt-get install -y openssl ca-certificates wget 
RUN wget https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb && apt-get install -y ./google-chrome-stable_current_amd64.deb
RUN apt-get install libxss1
RUN rm -rf /var/lib/apt/lists/*
COPY --from=taka_the_discord_bot_dependencies /app/build/taka_the_discord_bot_client .
COPY --from=taka_the_discord_bot_dependencies /app/taka_the_discord_bot_client/.env.prod ./.env
RUN update-ca-certificates
CMD ["/taka_the_discord_bot_client/taka_the_discord_bot_client"]