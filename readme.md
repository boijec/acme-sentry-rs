# ACME Sentry

A.C.M.E 2.0 client using the openssl bindings crate.
```
Rustc version: rustc 1.85.0 (4d91de4e4 2025-02-17)
```
ACME CA for Test: [Pebble](https://github.com/letsencrypt/pebble)

## Protocol
### Automated Certificate Management Environment version 2
The ACME2.0 protocol is the current version of the ACME protocol, a standard for automating
certificate issuance and management.  
ACME Sentry implements the client-side of RFC 8555.

A classic "happy-flow" case of how end users use the ACME protocol can be lifted from Certbot -
Let's Encrypt's ACME 2.0 client. For the sake of simplicity; "ACME Client" is Certbot and "ACME CA Server" is Let's Encrypt's
CA API.
```mermaid
sequenceDiagram
    actor A1 as System Administrator
    participant A as ACME Client
    A1 ->> A: Interact with client
    create participant B as ACME CA Server
    A ->> B: Authenticate
    B -->> A: Auth OK
    A ->> B: Certificate order
    B -->> A: OK
    A ->> B: Request challenge
    B -->> A: Challenge token
    A -->> A1: Challenge token 
    create participant C as DNS Provider / Web Server
    A1 ->> C: Add challenge token to DNS record or web server
    A1 ->> A: Ready for challenge!
    A ->> B: Ready for challenge!
    B -> C: Validate challenge token!
    B -->> A: Challenge success!
    destroy C
    A1 ->> C: Remove challenge token from DNS or web server
    A ->> B: Send CSR
    B -->> A: Certificates ready
    A ->> B: Fetch certificates
    B -->> A: Certificates in trust chain
    A -->> A1: Report status
```