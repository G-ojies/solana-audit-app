#!/usr/bin/env python3
"""
Ghost Protocol — Superteam Earn Autonomous Agent
GreYat Labs | @GreYat_Labs
--------------------------------------------------
Registers as an agent, discovers eligible listings,
filters by skill match, and submits work autonomously.
Human claims payout via claimCode.
"""

import os
import re
import json
import time
import logging
import argparse
from datetime import datetime, timezone
from typing import Optional
import requests

# Load .env if present (python-dotenv is optional but listed in requirements.txt)
try:
    from dotenv import load_dotenv
    # override=True so .env is the source of truth even if a stale key is
    # already exported in the shell (that mismatch causes silent 401s).
    load_dotenv(override=True)
except ImportError:
    pass

# ─── CONFIG ────────────────────────────────────────────────────────────────────

BASE_URL = "https://superteam.fun"
AGENT_NAME = "ghost-protocol"
TELEGRAM_URL = "http://t.me/G_Ojies"      # Human operator for project listings

# Network behaviour
HTTP_TIMEOUT = 15
MAX_RETRIES = 3
BACKOFF_BASE = 1.5  # seconds; doubles each retry

# Credentials — set via env or .env file (never hardcode)
API_KEY    = os.getenv("GHOST_API_KEY", "")
CLAIM_CODE = os.getenv("GHOST_CLAIM_CODE", "")
AGENT_ID   = os.getenv("GHOST_AGENT_ID", "")

# Skills Ghost Protocol matches on (keyword matching against listing titles/tags)
SKILL_KEYWORDS = [
    "solana", "rust", "anchor", "web3", "defi", "dapp",
    "react", "next.js", "nextjs", "typescript", "javascript",
    "python", "api", "backend", "frontend", "full-stack", "fullstack",
    "security", "audit", "smart contract", "blockchain", "crypto",
    "ai", "llm", "agent", "automation", "bot",
]

# Listing types to pursue
TARGET_TYPES = ["bounty", "project", "hackathon"]

# ─── LOGGING ──────────────────────────────────────────────────────────────────

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [GHOST] %(levelname)s — %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)
log = logging.getLogger("ghost-protocol")

# ─── HTTP CLIENT ───────────────────────────────────────────────────────────────

def auth_headers() -> dict:
    return {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json",
    }

def _retry_after(resp: requests.Response, attempt: int) -> float:
    """Honour a Retry-After header if present, else exponential backoff."""
    header = resp.headers.get("Retry-After") if resp is not None else None
    if header:
        try:
            return float(header)
        except ValueError:
            pass
    return BACKOFF_BASE * (2 ** attempt)

def _request(method: str, path: str, **kwargs) -> Optional[dict]:
    """Shared request helper with retries on 429 and 5xx, plus connection errors."""
    url = f"{BASE_URL}{path}"
    kwargs.setdefault("headers", auth_headers())
    kwargs.setdefault("timeout", HTTP_TIMEOUT)
    for attempt in range(MAX_RETRIES):
        try:
            r = requests.request(method, url, **kwargs)
            # Retry on rate-limit / transient server errors
            if r.status_code == 429 or 500 <= r.status_code < 600:
                if attempt < MAX_RETRIES - 1:
                    wait = _retry_after(r, attempt)
                    log.warning(f"{method} {path} → HTTP {r.status_code}, retrying in {wait:.1f}s "
                                f"({attempt + 1}/{MAX_RETRIES - 1})")
                    time.sleep(wait)
                    continue
            r.raise_for_status()
            if not r.content:
                return {}
            return r.json()
        except requests.exceptions.HTTPError as e:
            log.error(f"{method} {path} → HTTP {e.response.status_code}: {e.response.text[:200]}")
            return None
        except (requests.exceptions.ConnectionError, requests.exceptions.Timeout) as e:
            if attempt < MAX_RETRIES - 1:
                wait = BACKOFF_BASE * (2 ** attempt)
                log.warning(f"{method} {path} → {type(e).__name__}, retrying in {wait:.1f}s "
                            f"({attempt + 1}/{MAX_RETRIES - 1})")
                time.sleep(wait)
                continue
            log.error(f"{method} {path} → {e}")
        except Exception as e:
            log.error(f"{method} {path} → {e}")
            return None
    return None

def api_get(path: str, params: dict = None) -> Optional[dict]:
    return _request("GET", path, params=params)

def api_post(path: str, payload: dict) -> Optional[dict]:
    return _request("POST", path, json=payload)

# ─── REGISTRATION ──────────────────────────────────────────────────────────────

def register_agent() -> Optional[dict]:
    """Register Ghost Protocol as a new Earn agent."""
    log.info("Registering Ghost Protocol as a new Earn agent...")
    try:
        r = requests.post(
            f"{BASE_URL}/api/agents",
            headers={"Content-Type": "application/json"},
            json={"name": AGENT_NAME},
            timeout=15,
        )
        r.raise_for_status()
        data = r.json()
        log.info(f"✅ Registered: agentId={data.get('agentId')} | username={data.get('username')}")
        log.info(f"   API Key   : {data.get('apiKey')}")
        log.info(f"   Claim Code: {data.get('claimCode')}")
        log.warning("⚠️  Save these credentials — they won't be shown again!")
        return data
    except Exception as e:
        log.error(f"Registration failed: {e}")
        return None

# ─── HEARTBEAT ─────────────────────────────────────────────────────────────────

def emit_heartbeat(last_action: str = "idle", next_action: str = "scanning listings") -> dict:
    """Emit a heartbeat according to superteam.fun/heartbeat.md spec."""
    heartbeat = {
        "status": "ok",
        "agentName": AGENT_NAME,
        "time": datetime.now(timezone.utc).isoformat(),
        "version": "ghost-protocol-v1.0",
        "capabilities": ["register", "listings", "submit", "claim"],
        "lastAction": last_action,
        "nextAction": next_action,
    }
    log.info(f"💓 Heartbeat: {json.dumps(heartbeat)}")
    return heartbeat

# ─── LISTING DISCOVERY ─────────────────────────────────────────────────────────

def fetch_listings(take: int = 20) -> list[dict]:
    """Fetch all agent-eligible live listings."""
    # Try param combinations — Superteam's Prisma backend is strict
    param_sets = [
        {"take": take},
        {"take": str(take)},
        {},
    ]
    for params in param_sets:
        label = params or "no params"
        log.info(f"Fetching agent-eligible listings ({label})...")
        data = api_get("/api/agents/listings/live", params=params if params else None)
        if data is None:
            continue
        listings = data if isinstance(data, list) else data.get("listings", [])
        if listings:
            log.info(f"📋 Found {len(listings)} agent-eligible listings")
            return listings
        log.warning(f"Empty result with params {label}, trying next...")
    log.warning("No listings returned from any param set.")
    return []

def fetch_public_listings(take: int = 50) -> list[dict]:
    """Fetch the public Superteam Earn bounty board (no auth required).

    This is the human-submittable pool — much larger than the agent-only
    endpoint, and the place where genuinely-open opportunities show up.
    """
    log.info(f"Fetching public Superteam Earn listings (take={take})...")
    data = _request("GET", "/api/listings",
                    params={"take": take},
                    headers={"Content-Type": "application/json"})
    listings = data if isinstance(data, list) else (data or {}).get("listings", [])
    if listings:
        log.info(f"📋 Found {len(listings)} public listings")
    return listings or []

def safe_str(val) -> str:
    """Convert any value to a flat string for keyword matching."""
    if isinstance(val, list):
        return " ".join(safe_str(v) for v in val)
    if isinstance(val, dict):
        return " ".join(safe_str(v) for v in val.values())
    return str(val) if val is not None else ""

def score_listing(listing: dict) -> int:
    """Score a listing by skill keyword match. Higher = better fit."""
    score = 0
    text = " ".join([
        safe_str(listing.get("title", "")),
        safe_str(listing.get("description", "")),
        safe_str(listing.get("skills", "")),
        safe_str(listing.get("slug", "")),
        safe_str(listing.get("type", "")),
    ]).lower()
    for kw in SKILL_KEYWORDS:
        if kw in text:
            score += 1
    return score

def is_past_deadline(listing: dict) -> bool:
    """True if the listing's deadline has elapsed.

    The agent 'live listings' endpoint returns bounties as status=OPEN even
    after their deadline; submitting to those yields a 403 'Submissions closed'.
    Checking the deadline ourselves avoids wasting effort on dead listings.
    """
    deadline = listing.get("deadline")
    if not deadline:
        return False
    try:
        dt = datetime.fromisoformat(str(deadline).replace("Z", "+00:00"))
        return dt < datetime.now(timezone.utc)
    except (ValueError, TypeError):
        return False

def filter_listings(listings: list[dict], min_score: int = 1) -> list[dict]:
    """Filter and rank listings by skill match score, excluding closed listings."""
    scored = []
    closed = 0
    expired = 0
    for l in listings:
        if l.get("type") not in TARGET_TYPES:
            continue
        # Skip listings where submissions are closed
        if l.get("isSubmissionClosed") or l.get("status") == "CLOSED":
            closed += 1
            continue
        # Skip listings whose deadline has already passed (API still marks OPEN)
        if is_past_deadline(l):
            expired += 1
            continue
        s = score_listing(l)
        if s >= min_score:
            scored.append((s, l))
    if closed:
        log.info(f"⏭️  Skipped {closed} listings with closed submissions")
    if expired:
        log.info(f"⏭️  Skipped {expired} listings past their deadline")
    scored.sort(key=lambda x: x[0], reverse=True)
    result = [l for _, l in scored]
    log.info(f"🎯 Filtered to {len(result)} open, skill-matched listings")
    return result

def get_listing_details(slug: str) -> Optional[dict]:
    """Fetch full details for a specific listing."""
    return api_get(f"/api/agents/listings/details/{slug}")

# ─── SUBMISSION ────────────────────────────────────────────────────────────────

def strip_html(html: str) -> str:
    """Crude HTML→text for reading listing descriptions in the terminal."""
    text = re.sub(r"<[^>]+>", " ", html or "")
    text = re.sub(r"\s+", " ", text)
    return text.strip()

def draft_description(listing: dict, link: str) -> str:
    """Generate a default submission write-up when none is supplied."""
    title = listing.get("title", "this listing")
    return (
        f"Submission by Ghost Protocol (autonomous Superteam Earn agent) for \"{title}\".\n\n"
        f"Deliverable: {link}\n\n"
        "Approach: the linked work addresses the listing requirements end-to-end. "
        "Full context, reproduction steps, and any proofs are documented at the link above. "
        f"For coordination, reach the human operator at {TELEGRAM_URL}."
    )

def eligibility_questions(listing: dict) -> list[dict]:
    """Normalise the listing's eligibility questions to a list of dicts."""
    return listing.get("eligibilityQuestions") or listing.get("eligibility") or []

def build_submission(listing: dict, link: str, description: str,
                     ask: float = None, answers: dict = None) -> dict:
    """Build a submission payload for a listing.

    `answers` maps question text → answer string; any question left unanswered
    is included with an empty answer (the caller is warned).
    """
    answers = answers or {}
    payload = {
        "listingId": listing["id"],
        "link": link,
        "tweet": "",
        "otherInfo": description,
        "eligibilityAnswers": [],
        "ask": ask,
        "telegram": TELEGRAM_URL,
    }

    questions = eligibility_questions(listing)
    if questions:
        built = []
        unanswered = 0
        for q in questions:
            qtext = q.get("question", "") if isinstance(q, dict) else str(q)
            ans = answers.get(qtext, "")
            if not ans:
                unanswered += 1
            built.append({"question": qtext, "answer": ans})
        payload["eligibilityAnswers"] = built
        if unanswered:
            log.warning(f"⚠️  {unanswered}/{len(questions)} eligibility questions are unanswered.")

    return payload

def parse_answer_args(listing: dict, raw_answers: list) -> dict:
    """Resolve `--answer key=value` pairs to {question_text: answer}.

    `key` may be the question's 1-based index, its exact text, or a
    case-insensitive substring of the question. Unmatched or ambiguous keys
    are logged and skipped.
    """
    answers = {}
    if not raw_answers:
        return answers
    questions = [
        (q.get("question", "") if isinstance(q, dict) else str(q))
        for q in eligibility_questions(listing)
    ]
    for item in raw_answers:
        if "=" not in item:
            log.warning(f"⚠️  Ignoring --answer '{item}' (expected key=value).")
            continue
        key, value = item.split("=", 1)
        key, value = key.strip(), value.strip()

        # 1) index match
        if key.isdigit():
            idx = int(key) - 1
            if 0 <= idx < len(questions):
                answers[questions[idx]] = value
            else:
                log.warning(f"⚠️  --answer index {key} out of range (1..{len(questions)}).")
            continue
        # 2) exact match
        if key in questions:
            answers[key] = value
            continue
        # 3) substring match (must be unambiguous)
        matches = [q for q in questions if key.lower() in q.lower()]
        if len(matches) == 1:
            answers[matches[0]] = value
        elif len(matches) > 1:
            log.warning(f"⚠️  --answer key '{key}' matches {len(matches)} questions; "
                        f"use the exact text or index.")
        else:
            log.warning(f"⚠️  --answer key '{key}' matched no eligibility question.")
    return answers

def submit_listing(payload: dict) -> Optional[dict]:
    """Submit work for a listing."""
    log.info(f"📤 Submitting to listing {payload['listingId']}...")
    result = api_post("/api/agents/submissions/create", payload)
    if result:
        log.info("✅ Submission successful!")
    return result

def update_submission(payload: dict) -> Optional[dict]:
    """Update an existing submission."""
    log.info(f"🔄 Updating submission for listing {payload['listingId']}...")
    return api_post("/api/agents/submissions/update", payload)

# ─── COMMENTS ─────────────────────────────────────────────────────────────────

def get_comments(listing_id: str, skip: int = 0, take: int = 20) -> list[dict]:
    """Fetch comments for a listing."""
    data = api_get(f"/api/agents/comments/{listing_id}", params={"skip": skip, "take": take})
    return data if isinstance(data, list) else []

def post_comment(listing_id: str, message: str, poc_id: str, ref_type: str = "BOUNTY") -> Optional[dict]:
    """Post a comment on a listing."""
    return api_post("/api/agents/comments/create", {
        "refType": ref_type,
        "refId": listing_id,
        "message": message,
        "pocId": poc_id,
    })

# ─── DISPLAY ───────────────────────────────────────────────────────────────────

WIDTH = 90

def print_listings_table(listings: list[dict]) -> None:
    """Render a summary table of listings (shared by scan and show)."""
    print("\n" + "═" * WIDTH)
    print(f"{'GHOST PROTOCOL — LISTINGS':^{WIDTH}}")
    print("═" * WIDTH)
    print(f"{'#':<4} {'SCORE':<7} {'TYPE':<12} {'REWARD':<14} {'ACCESS':<16} {'TITLE'}")
    print("─" * WIDTH)
    for i, l in enumerate(listings, 1):
        title = l.get("title", "Untitled")[:34]
        ltype = l.get("type", "?").upper()
        reward = l.get("rewardAmount") or l.get("totalRewardAmount") or "?"
        token = l.get("token", "")
        score = l.get("_skillScore", score_listing(l))
        access = l.get("agentAccess", "?")
        access_label = "🤖 AGENT_ONLY" if access == "AGENT_ONLY" else "✅ ALLOWED"
        print(f"{i:<4} {score:<7} {ltype:<12} {str(reward) + ' ' + token:<14} {access_label:<16} {title}")
    print("═" * WIDTH + "\n")

def load_saved_listings(path: str) -> list[dict]:
    """Load previously scanned listings from a JSON file."""
    if not os.path.exists(path):
        log.error(f"No saved listings at {path}. Run a scan first.")
        return []
    try:
        with open(path) as f:
            data = json.load(f)
        return data if isinstance(data, list) else []
    except (json.JSONDecodeError, OSError) as e:
        log.error(f"Could not read {path}: {e}")
        return []

# ─── SCAN LOOP ─────────────────────────────────────────────────────────────────

def scan_and_report(min_score: int = 2, output_file: str = "ghost_listings.json") -> list[dict]:
    """
    Main discovery loop:
    1. Fetch all agent-eligible listings
    2. Score and filter by skill match
    3. Enrich with full details
    4. Save to JSON for review
    """
    emit_heartbeat(last_action="starting scan", next_action="fetching listings")

    raw = fetch_listings(take=20)
    if not raw:
        log.warning("No listings returned. Check API key or Superteam Earn status.")
        return []

    filtered = filter_listings(raw, min_score=min_score)

    enriched = []
    for listing in filtered[:20]:  # cap at 20 to avoid rate limits
        slug = listing.get("slug", "")
        if slug:
            details = get_listing_details(slug)
            if details:
                listing.update(details)
        listing["_skillScore"] = score_listing(listing)
        enriched.append(listing)
        time.sleep(0.5)  # polite pacing

    # Save report
    with open(output_file, "w") as f:
        json.dump(enriched, f, indent=2, default=str)
    log.info(f"💾 Saved {len(enriched)} listings to {output_file}")

    print_listings_table(enriched)

    emit_heartbeat(
        last_action=f"scanned {len(enriched)} listings",
        next_action="awaiting human review or auto-submit"
    )
    return enriched

def watch(min_score: int = 2, public: bool = True) -> list[dict]:
    """Lightweight discovery for scheduled runs.

    Fetches live listings from the agent-eligible endpoint and (by default) the
    public Superteam Earn bounty board, merges + dedupes them, drops closed /
    past-deadline / low-match ones, and prints a machine-greppable verdict.
    Designed to be run on a schedule: a wrapping agent notifies the human only
    when OPEN matches appear.

    Prints a line beginning with 'GHOST_WATCH:' summarising the result.
    """
    raw = list(fetch_listings(take=20) or [])
    for l in raw:
        l.setdefault("_source", "agent")

    if public:
        for l in fetch_public_listings(take=50):
            l["_source"] = "public"
            raw.append(l)

    # Dedupe by listing id (agent + public can overlap), keeping the first seen.
    seen = set()
    deduped = []
    for l in raw:
        lid = l.get("id")
        if lid in seen:
            continue
        seen.add(lid)
        deduped.append(l)

    open_matches = filter_listings(deduped, min_score=min_score) if deduped else []

    if open_matches:
        print(f"GHOST_WATCH: {len(open_matches)} OPEN skill-matched listing(s) found 🔔")
        print_listings_table(open_matches)
        for l in open_matches:
            print(f"  • [{l.get('_skillScore', score_listing(l))}] "
                  f"{l.get('title', 'Untitled')} — {l.get('rewardAmount')} {l.get('token', '')} "
                  f"| {l.get('_source', '?')} | slug={l.get('slug')} | deadline={l.get('deadline')}")
    else:
        print("GHOST_WATCH: 0 open listings (nothing to submit to right now).")
    return open_matches

# ─── COMMAND HANDLERS ────────────────────────────────────────────────────────

def resolve_listing(args) -> Optional[dict]:
    """Resolve the target listing for submit, from --slug or --from-saved index."""
    if args.from_saved is not None:
        saved = load_saved_listings(args.output)
        if not saved:
            return None
        idx = args.from_saved - 1  # 1-based to match the table
        if idx < 0 or idx >= len(saved):
            log.error(f"--from-saved {args.from_saved} out of range (1..{len(saved)}).")
            return None
        listing = saved[idx]
        slug = listing.get("slug")
        # Refresh details so id/questions are current
        details = get_listing_details(slug) if slug else None
        if details:
            listing.update(details)
        return listing
    if args.slug:
        details = get_listing_details(args.slug)
        if not details:
            log.error(f"Could not fetch listing: {args.slug}")
        return details
    log.error("submit requires either --slug or --from-saved N.")
    return None

def cmd_submit(args) -> None:
    if not API_KEY:
        log.error("GHOST_API_KEY not set. Export it or add to .env")
        return
    if not args.link:
        log.error("submit requires --link (URL to your deliverable).")
        return

    listing = resolve_listing(args)
    if not listing:
        return

    # Show what we're submitting to
    print("\n" + "─" * WIDTH)
    print(f"TARGET: {listing.get('title', '?')}")
    print(f"  type={listing.get('type')}  reward={listing.get('rewardAmount')} {listing.get('token', '')}  "
          f"access={listing.get('agentAccess', '?')}")
    desc = strip_html(listing.get("description", ""))
    if desc:
        print(f"  brief: {desc[:300]}{'…' if len(desc) > 300 else ''}")
    print("─" * WIDTH)

    description = args.info or draft_description(listing, args.link)
    if not args.info:
        log.info("No --info supplied; using an auto-generated draft description.")

    # Eligibility answers: start from --answer flags, then fall back to
    # interactively prompting for anything still missing (unless --yes).
    answers = parse_answer_args(listing, args.answer)
    questions = eligibility_questions(listing)
    if questions and not args.yes:
        missing = [
            (q.get("question", "") if isinstance(q, dict) else str(q))
            for q in questions
        ]
        missing = [q for q in missing if not answers.get(q)]
        if missing:
            print(f"\n📝 {len(missing)} eligibility question(s) still need answers "
                  f"(blank = leave empty):")
            for q in missing:
                ans = input(f"  • {q}\n     > ").strip()
                if ans:
                    answers[q] = ans

    payload = build_submission(listing, args.link, description, ask=args.ask, answers=answers)

    print("\n📦 Submission payload:")
    print(json.dumps(payload, indent=2))

    if args.dry_run:
        log.info("🧪 --dry-run set: nothing was submitted.")
        return

    if args.yes:
        submit_listing(payload)
        return

    confirm = input("\n▶ Submit this to Superteam Earn? [y/N] ").strip().lower()
    if confirm == "y":
        submit_listing(payload)
    else:
        log.info("Submission cancelled.")

# ─── CLI ───────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(
        description="Ghost Protocol — Superteam Earn Agent",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Commands:
  register          Register a new agent identity
  scan              Discover and score agent-eligible listings
  show              Print the table from a previous scan (no network)
  heartbeat         Emit a status heartbeat
  submit            Submit to a listing (guarded; --dry-run to preview)

Examples:
  python ghost_protocol.py scan --min-score 3
  python ghost_protocol.py show
  python ghost_protocol.py submit --from-saved 2 --link https://github.com/me/pr --dry-run
  python ghost_protocol.py submit --from-saved 2 --link https://… \\
      --answer 1=https://youtu.be/pitch --answer repo=https://github.com/me/x --yes
  python ghost_protocol.py submit --slug some-bounty --link https://… --info "..." --yes
        """,
    )
    parser.add_argument("command", choices=["register", "scan", "show", "watch", "heartbeat", "submit"])
    parser.add_argument("--min-score", type=int, default=2, help="Minimum skill match score (default: 2)")
    parser.add_argument("--output", default="ghost_listings.json", help="JSON file for scan results / show / --from-saved")
    parser.add_argument("--slug", help="Listing slug for submit")
    parser.add_argument("--from-saved", type=int, metavar="N", help="Submit to row N from the saved scan (see 'show')")
    parser.add_argument("--link", help="Submission link URL")
    parser.add_argument("--info", help="Submission description / otherInfo (auto-drafted if omitted)")
    parser.add_argument("--answer", action="append", default=[], metavar="KEY=VALUE",
                        help="Eligibility answer; KEY is the question index, exact text, or a "
                             "substring. Repeatable, e.g. --answer 1=https://… --answer repo=https://…")
    parser.add_argument("--ask", type=float, help="Reward ask amount (for variable-comp listings)")
    parser.add_argument("--dry-run", action="store_true", help="Build and print the payload without submitting")
    parser.add_argument("--yes", action="store_true", help="Skip the confirmation prompt (non-interactive)")
    parser.add_argument("--no-public", action="store_true", help="watch: only the agent endpoint, skip the public bounty board")

    args = parser.parse_args()

    if args.command == "register":
        register_agent()

    elif args.command == "scan":
        if not API_KEY:
            log.error("GHOST_API_KEY not set. Export it or add to .env")
            return
        scan_and_report(min_score=args.min_score, output_file=args.output)

    elif args.command == "show":
        saved = load_saved_listings(args.output)
        if saved:
            print_listings_table(saved)

    elif args.command == "watch":
        watch(min_score=args.min_score, public=not args.no_public)

    elif args.command == "heartbeat":
        emit_heartbeat()

    elif args.command == "submit":
        cmd_submit(args)

if __name__ == "__main__":
    main()
