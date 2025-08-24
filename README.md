# HTTP-Mailer

A remarkably simple and lightweight HTTP API for sending emails.  
Emails are sent through a local SMTP server at port 25 (tested with Postfix).

## CLI documentation

```bash
http-mailer 0.2.0
Mubelotix <mubelotix@gmail.com>

USAGE:
    http-mailer [OPTIONS]

OPTIONS:
    -a, --addr <ADDR>          Address to listen on [default: localhost:8000]
    -h, --help                 Print help information
    -k, --api-key <API_KEY>    Hashed api key (with sha256)
    -V, --version              Print version information
```

## HTTP example

```http
POST /send-email HTTP/2
Api-Key: password
From: origin@insagenda.fr
To: destination@example.org
Reply-To: someone@gmail.com
Subject: Testing email

This is the body
```

## HTTP example for multipart of text+html

```http
POST /send-email HTTP/2
Api-Key: password
From: origin@insagenda.fr
To: destination@example.org
Reply-To: someone@gmail.com
Subject: Testing email

This is a text message.
-----END-TEXT-BEGIN-HTML-----
<p>This is a text <i>message<i>.<p>
```
