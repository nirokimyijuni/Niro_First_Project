"""에이전트 대시보드 서버. 표준 라이브러리만 쓴다.

실행: python tools/dashboard/server.py  →  http://127.0.0.1:4100
- /              대시보드 화면 (index.html)
- /api/activity  활동 기록의 마지막 줄들 (JSON 배열)
- /api/roadmap   마일스톤별 스토리 진행 상황 (docs/02-plan.md 표 + stories/backlog.md)
- /font.ttf, /characters.png  게임 에셋을 그대로 보여 준다
"""
import json
import re
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
PORT = 4100
TAIL_LINES = 600

FILES = {
    "/": (HERE / "index.html", "text/html; charset=utf-8"),
    "/font.ttf": (ROOT / "game" / "assets" / "fonts" / "Jua-Regular.ttf", "font/ttf"),
    "/characters.png": (ROOT / "game" / "assets" / "sprites" / "characters.png", "image/png"),
}


def read_tail():
    try:
        lines = LOG_FILE.read_text(encoding="utf-8").splitlines()[-TAIL_LINES:]
    except FileNotFoundError:
        return []
    records = []
    for line in lines:
        try:
            records.append(json.loads(line))
        except ValueError:
            pass
    return records


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
