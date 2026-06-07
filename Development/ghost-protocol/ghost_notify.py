#!/usr/bin/env python3
"""
Ghost Protocol notifier.

Sends an alert via Telegram (preferred) or SMTP email — whichever is configured
in the environment / .env. Reads the message body from --body or stdin. Used by
ghost_watch.sh to alert when open listings appear.

Telegram (recommended) — add to .env:
  TELEGRAM_BOT_TOKEN   from @BotFather
  TELEGRAM_CHAT_ID     your chat id (use `--telegram-chats` to discover it)

Email (fallback) — add to .env:
  SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS
  ALERT_EMAIL_FROM (default SMTP_USER), ALERT_EMAIL_TO (default SMTP_USER)

Usage:
  python3 ghost_notify.py --subject "..." --body "..."
  echo "body" | python3 ghost_notify.py --subject "..."
  python3 ghost_notify.py --test              # send a test alert
  python3 ghost_notify.py --telegram-chats    # list chat ids that messaged the bot
"""
import os
import sys
import ssl
import smtplib
import argparse
from email.message import EmailMessage

import requests

try:
    from dotenv import load_dotenv
    load_dotenv(override=True)
except ImportError:
    pass


def _telegram_configured() -> bool:
    return bool(os.getenv("TELEGRAM_BOT_TOKEN") and os.getenv("TELEGRAM_CHAT_ID"))


def _smtp_configured() -> bool:
    return bool(os.getenv("SMTP_HOST") and os.getenv("SMTP_USER") and os.getenv("SMTP_PASS"))


def send_telegram(subject: str, body: str) -> int:
    token = os.getenv("TELEGRAM_BOT_TOKEN", "")
    chat_id = os.getenv("TELEGRAM_CHAT_ID", "")
    # Plain text (no parse_mode) so URLs / underscores never trigger a 400.
    text = f"{subject}\n\n{body}" if subject else body
    try:
        r = requests.post(
            f"https://api.telegram.org/bot{token}/sendMessage",
            json={"chat_id": chat_id, "text": text,
                  "disable_web_page_preview": True},
            timeout=30,
        )
        if r.status_code != 200:
            print(f"[ghost_notify] Telegram failed: HTTP {r.status_code} {r.text[:200]}",
                  file=sys.stderr)
            return 1
    except Exception as e:
        print(f"[ghost_notify] Telegram failed: {e}", file=sys.stderr)
        return 1
    print(f"[ghost_notify] Telegram alert sent to chat {chat_id}", file=sys.stderr)
    return 0


def list_telegram_chats() -> int:
    token = os.getenv("TELEGRAM_BOT_TOKEN", "")
    if not token:
        print("[ghost_notify] TELEGRAM_BOT_TOKEN not set in .env", file=sys.stderr)
        return 2
    try:
        r = requests.get(f"https://api.telegram.org/bot{token}/getUpdates", timeout=30)
        data = r.json()
    except Exception as e:
        print(f"[ghost_notify] getUpdates failed: {e}", file=sys.stderr)
        return 1
    if not data.get("ok"):
        print(f"[ghost_notify] getUpdates error: {data}", file=sys.stderr)
        return 1
    seen = {}
    for upd in data.get("result", []):
        msg = upd.get("message") or upd.get("channel_post") or {}
        chat = msg.get("chat", {})
        if chat.get("id") is not None:
            who = chat.get("username") or chat.get("title") or \
                f"{chat.get('first_name', '')} {chat.get('last_name', '')}".strip()
            seen[chat["id"]] = f"{who} ({chat.get('type')})"
    if not seen:
        print("No chats found. Send a message to your bot first, then re-run.",
              file=sys.stderr)
        return 1
    print("Chat IDs that have messaged the bot:")
    for cid, who in seen.items():
        print(f"  TELEGRAM_CHAT_ID={cid}   # {who}")
    return 0


def send_email(subject: str, body: str) -> int:
    host = os.getenv("SMTP_HOST", "")
    port = int(os.getenv("SMTP_PORT", "587"))
    user = os.getenv("SMTP_USER", "")
    password = os.getenv("SMTP_PASS", "")
    sender = os.getenv("ALERT_EMAIL_FROM", user)
    recipient = os.getenv("ALERT_EMAIL_TO", user)

    msg = EmailMessage()
    msg["Subject"] = subject
    msg["From"] = sender
    msg["To"] = recipient
    msg.set_content(body)

    try:
        if port == 465:
            with smtplib.SMTP_SSL(host, port, timeout=30,
                                  context=ssl.create_default_context()) as s:
                s.login(user, password)
                s.send_message(msg)
        else:
            with smtplib.SMTP(host, port, timeout=30) as s:
                s.ehlo()
                s.starttls(context=ssl.create_default_context())
                s.login(user, password)
                s.send_message(msg)
    except Exception as e:
        print(f"[ghost_notify] Email send failed: {e}", file=sys.stderr)
        return 1
    print(f"[ghost_notify] Email sent to {recipient}", file=sys.stderr)
    return 0


def notify(subject: str, body: str) -> int:
    """Send via Telegram if configured, else email. Returns exit code."""
    if _telegram_configured():
        return send_telegram(subject, body)
    if _smtp_configured():
        return send_email(subject, body)
    print("[ghost_notify] No notifier configured (set TELEGRAM_* or SMTP_* in .env). "
          "Nothing sent.", file=sys.stderr)
    return 2


def main() -> int:
    p = argparse.ArgumentParser(description="Ghost Protocol notifier (Telegram/email)")
    p.add_argument("--subject", default="Ghost Protocol alert")
    p.add_argument("--body", help="Message body (else read from stdin)")
    p.add_argument("--test", action="store_true", help="Send a test alert")
    p.add_argument("--telegram-chats", action="store_true",
                   help="List chat ids that have messaged the bot")
    args = p.parse_args()

    if args.telegram_chats:
        return list_telegram_chats()
    if args.test:
        return notify("Ghost Protocol — test alert",
                      "If you received this, alerts are working. 👻")

    body = args.body if args.body is not None else sys.stdin.read()
    return notify(args.subject, body)


if __name__ == "__main__":
    sys.exit(main())
