FROM unsealer AS unsealer

FROM node:lts

COPY --from=unsealer /opt/unsealer /opt/unsealer

COPY server.js package.json ./
RUN npm install

ENTRYPOINT ["/opt/unsealer/unsealer"]
