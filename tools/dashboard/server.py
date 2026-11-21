"""에이전트 대시보드 서버. 표준 라이브러리만 쓴다.

실행: python tools/dashboard/server.py  →  http://127.0.0.1:4100
- /              대시보드 화면 (index.html)
- /api/activity  활동 기록의 마지막 줄들 (JSON 배열)
- /api/today     오늘 한 일 요약 (에이전트별 시각·호출 수·많이 만진 파일, 오늘 커밋)
- /api/roadmap   마일스톤별 스토리 진행 상황 (docs/02-plan.md 표 + stories/backlog.md)
- /font.ttf, /<이름>.png  게임 에셋을 그대로 보여 준다 (에이전트 프로필 그림)
"""
import json
import os
import re
import subprocess
import time
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
LOG_FILE = ROOT / ".claude" / "activity" / "activity.jsonl"
PLAN_FILE = ROOT / "docs" / "02-plan.md"
BACKLOG_FILE = ROOT / "stories" / "backlog.md"
# 계획 표의 한 줄: | M2 | 낮과 밤 | 내용 | 아이와 함께 할 일 |
MILESTONE_ROW = re.compile(r"^\|\s*(M\d+)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|")
# 스토리 제목 줄: ### US-009 제목  [M2] [할 일]
STORY_LINE = re.compile(r"^###\s+(US-\d+)\s+(.+?)\s+\[(M\d+)\]\s+\[([^\]]+)\]")
PORT = int(os.environ.get("DASH_PORT", 4100))  # 시험할 때는 DASH_PORT로 다른 포트를 쓴다
TAIL_LINES = 600

# ---- 개인 정보 가리기 (순수 함수) ----
# 화면에는 프로젝트 폴더 아래 상대 경로만 보인다. 밖의 경로는 "(프로젝트 밖)".
OUTSIDE = "(프로젝트 밖)"
# 드라이브 경로(C:\..., C:/...) 또는 msys 경로(/c/...). 뒤는 공백·따옴표 전까지 가린다.
ABS_PATH = re.compile(r"(?:[A-Za-z]:[\\/]|(?<![\w.])/[A-Za-z]/)[^\s\"'|;&<>]*")


def leftover_pattern(root):
    """마지막 안전망: 루트 바로 위 폴더 이름(사람 이름이 들어 있을 수 있음)이 어떤 이유로든 남으면 지운다.
    이름을 코드에 적지 않으려고 루트 경로에서 읽는다."""
    parent = Path(str(root).replace("\\", "/")).parent.name
    return re.compile(re.escape(parent)) if parent and not parent.endswith(":") else None


def root_pattern(root):
    """프로젝트 루트를 역슬래시, 슬래시, msys(/d/...) 어느 모양으로 써도 잡는 정규식."""
    parts = str(root).replace("\\", "/").split("/")
    drive = parts[0]  # 예: "D:"
    rest = [re.escape(p) for p in parts[1:] if p]
    head = r"(?:" + re.escape(drive) + r"|/" + re.escape(drive[:1]) + r")" if drive.endswith(":") else re.escape(drive)
    return re.compile(head + "".join(r"[\\/]" + p for p in rest) + r"(?P<sep>[\\/]?)", re.I)


def mask_text(text, root=ROOT):
    """글 속의 절대 경로를 가린다. 루트 아래는 상대 경로로, 밖은 (프로젝트 밖)."""
    if not isinstance(text, str):
        return text
    text = root_pattern(root).sub(lambda m: "" if m.group("sep") else ".", text)
    text = ABS_PATH.sub(OUTSIDE, text)
    left = leftover_pattern(root)
    return left.sub("", text) if left else text


def mask_value(v, root=ROOT):
    if isinstance(v, str):
        return mask_text(v, root)
    if isinstance(v, dict):
        return {k: mask_value(x, root) for k, x in v.items()}
    if isinstance(v, list):
        return [mask_value(x, root) for x in v]
    return v


PUBLIC_FIELDS = ("ts", "event", "agent_id", "agent_type", "tool", "input", "error")


def clean_record(r, root=ROOT):
    """화면으로 내보낼 기록: 필요한 칸만 남기고(세션·기록 파일 경로 등은 버림) 경로를 가린다."""
    return {k: mask_value(r[k], root) for k in PUBLIC_FIELDS if k in r}


# ---- 오늘 한 일 ----
ALIASES = {"joshua": "susanna"}  # index.html의 ALIAS와 같다
READ_TOOLS = {"Read", "Grep", "Glob"}
EDIT_TOOLS = {"Edit", "Write", "NotebookEdit"}
RUN_TOOLS = {"Bash", "PowerShell"}


def agent_of(r):
    a = r.get("agent_type") or "main"
    return ALIASES.get(a, a)


def summarize_today(records, day_start, day_end, root=ROOT):
    """오늘(day_start 이상 day_end 미만) 기록을 에이전트별로 모은다. 순수 함수."""
    agents = {}
    for r in records:
        ts = r.get("ts")
        if not isinstance(ts, (int, float)) or not day_start <= ts < day_end:
            continue
        a = agents.setdefault(agent_of(r), {"first": ts, "last": ts, "lines": 0, "calls": 0,
                                           "read": 0, "edit": 0, "run": 0, "other": 0, "files": {}})
        a["first"], a["last"] = min(a["first"], ts), max(a["last"], ts)
        a["lines"] += 1
        if r.get("event") != "PreToolUse":
            continue  # 도구 한 번 = PreToolUse 한 줄
        tool = r.get("tool") or ""
        a["calls"] += 1
        kind = ("read" if tool in READ_TOOLS else "edit" if tool in EDIT_TOOLS
                else "run" if tool in RUN_TOOLS else "other")
        a[kind] += 1
        fp = (r.get("input") or {}).get("file_path")
        if fp and tool in READ_TOOLS | EDIT_TOOLS:
            shown = mask_text(fp, root).replace("\\", "/")
            if OUTSIDE not in shown:
                a["files"][shown] = a["files"].get(shown, 0) + 1
    for a in agents.values():
        top = sorted(a.pop("files").items(), key=lambda kv: (-kv[1], kv[0]))[:5]
        a["top_files"] = [{"path": p, "count": c} for p, c in top]
    return agents


def today_commits(day_start):
    """오늘 커밋 한 줄씩 (git log --since). git이 없거나 실패하면 빈 목록."""
    try:
        since = time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(day_start))
        out = subprocess.run(["git", "log", f"--since={since}", "--format=%h %s", "-n", "12"],
                             cwd=ROOT, capture_output=True, timeout=5, check=True).stdout
        return [mask_text(l) for l in out.decode("utf-8", "replace").splitlines() if l.strip()]
    except Exception:
        return []


def read_today():
    now = time.time()
    lt = time.localtime(now)
    day_start = time.mktime((lt.tm_year, lt.tm_mon, lt.tm_mday, 0, 0, 0, 0, 0, -1))
    agents = summarize_today(read_all(), day_start, day_start + 86400)
    return {"date": time.strftime("%Y-%m-%d", lt), "agents": agents,
            "active_agents": len(agents), "total_lines": sum(a["lines"] for a in agents.values()),
            "total_calls": sum(a["calls"] for a in agents.values()),
            "commits": today_commits(day_start)}


FILES = {
    "/": (HERE / "index.html", "text/html; charset=utf-8"),
    "/font.ttf": (ROOT / "game" / "assets" / "fonts" / "Jua-Regular.ttf", "font/ttf"),
    **{f"/{n}.png": (ROOT / "art" / "spritecook" / f"{n}.png", "image/png")
       for n in ("niro", "ellie", "logan", "zombie-round", "skeleton-normal", "skeleton-full")},
}


def read_all(limit=None):
    try:
        lines = LOG_FILE.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError:
        return []
    records = []
    for line in (lines[-limit:] if limit else lines):
        try:
            records.append(json.loads(line))
        except ValueError:
            pass
    return records


def read_tail():
    return [clean_record(r) for r in read_all(TAIL_LINES)]


def read_roadmap():
    def lines(file):
        try:
            return file.read_text(encoding="utf-8").splitlines()
        except FileNotFoundError:
            return []

    milestones, seen = [], set()
    for line in lines(PLAN_FILE):
        m = MILESTONE_ROW.match(line)
        if m and m.group(1) not in seen:
            seen.add(m.group(1))
            milestones.append({"id": m.group(1), "name": m.group(2), "summary": m.group(3), "stories": []})
    by_id = {m["id"]: m for m in milestones}
    for line in lines(BACKLOG_FILE):
        s = STORY_LINE.match(line)
        if s and s.group(3) in by_id:
            by_id[s.group(3)]["stories"].append({"id": s.group(1), "title": s.group(2), "status": s.group(4)})
    return milestones


class Handler(SimpleHTTPRequestHandler):
    def do_GET(self):
        path = self.path.split("?")[0]
        if path == "/api/activity":
            self.send(json.dumps(read_tail(), ensure_ascii=False).encode("utf-8"),
                      "application/json; charset=utf-8")
        elif path == "/api/today":
            self.send(json.dumps(read_today(), ensure_ascii=False).encode("utf-8"),
                      "application/json; charset=utf-8")
        elif path == "/api/roadmap":
            self.send(json.dumps(read_roadmap(), ensure_ascii=False).encode("utf-8"),
                      "application/json; charset=utf-8")
        elif path in FILES:
            file, ctype = FILES[path]
            self.send(file.read_bytes(), ctype)
        else:
            self.send_error(404)

    def send(self, body, ctype):
        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass  # 1초마다 오는 요청 기록은 끄기


if __name__ == "__main__":
    print(f"대시보드: http://127.0.0.1:{PORT}")
    ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
