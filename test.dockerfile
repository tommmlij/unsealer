FROM unsealer AS unsealer

FROM node:lts

COPY --from=unsealer /opt/unseal /opt/unseal

#ENTRYPOINT ["/opt/unseal/bin/python"]
