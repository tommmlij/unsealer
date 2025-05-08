FROM python:3.12-slim AS unsealer

WORKDIR /opt/unseal

COPY src/run.py src/requirements.txt /opt/unseal/

RUN  pip install --upgrade pip \
 && pip install --no-cache-dir virtualenv \
 && virtualenv /opt/unseal --copies \
 && /opt/unseal/bin/pip install --upgrade pip \
 && /opt/unseal/bin/pip install -r requirements.txt

CMD ["/opt/unseal/bin/python", "run.py"]