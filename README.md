# 🔐 Unsealer

A minimal runtime secrets unsealing service for containerized applications.

This project provides a lightweight Axum-based HTTP server designed to **securely inject secrets** (such as API keys or configuration) into containerized services **at runtime**, without the need for heavy infrastructure like Vault.

## 🚀 How It Works

1. The unsealer exposes two endpoints: `POST /init` and a health route `GET /health` 
2. You send a **NaCl-encrypted payload** as base64url via the `config` field in a JSON body.
3. The server decrypts the payload using its private key and your public key.
4. It spawns the target service setting individual keys as **environment variables**

---

## 📦 Environment Variables

| Variable             | Description                                                                                 |
|----------------------|---------------------------------------------------------------------------------------------|
| `PORT`               | Port to bind the HTTP server (default: `3000`)                                              |
| `SERVER_PRIVATE_KEY` | Base64-encoded NaCl private key of the unsealer                                             |
| `MANAGER_PUBLIC_KEY` | Base64-encoded NaCl public key of the client who wants to unseal                            |
| `COMMAND`            | Command to run (e.g. `npm run dev` or `echo ${SECRET_HELLO_WORLD}`) after successful unseal |

---

## 🧾 Request Format

**Endpoint**: `POST /init`  
**Content-Type**: `application/json`

```json
{
  "config": "<base64url(NONCE + CIPHERTEXT)>"
}
```

- The `config` must be encrypted using NaCl `Box`, with:
    - The **client's private key**
    - The **unsealer's public key**

---

## 🧪 Example: Sending a Request

```bash
curl -X POST http://localhost:3000/init?format=json \
  -H "Content-Type: application/json" \
  -d '{"config": "<base64url-encoded-data>"}'
```

---

## 🔐 Key Management

Keys are 32-byte NaCl keypairs encoded in base64.

To generate compatible keys using PyNaCl:

```python
from nacl.public import PrivateKey
import base64

sk = PrivateKey.generate()
pk = sk.public_key

print("Private Key:", base64.b64encode(sk.encode()).decode())
print("Public Key: ", base64.b64encode(pk.encode()).decode())
```

---

## 🧰 Building

```bash
cargo build --release
```

This produces a single static `unsealer` binary you can embed in any container image.

---

## 🐳 Docker Usage

```Dockerfile
FROM node:lts
COPY unsealer /usr/local/bin/unsealer
CMD ["unsealer"]
```

---

## 🛡️ Security Notes

- All secrets must be encrypted on the client side before transmission.
- Payloads should be short-lived or one-time-use to prevent replay attacks (future enhancement).
- Consider HTTPS or reverse proxy termination in production.

---

## 📄 License

MIT
