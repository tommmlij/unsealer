FROM unsealer AS unsealer

FROM python:alpine

COPY --from=unsealer /opt/unsealer /opt/unsealer

COPY app.py ./
RUN pip install aiohttp

ENTRYPOINT ["/opt/unsealer/unsealer"]
