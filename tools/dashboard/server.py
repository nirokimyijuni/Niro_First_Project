"""에이전트 대시보드 서버. 표준 라이브러리만 쓴다.

실행: python tools/dashboard/server.py  →  http://127.0.0.1:4100
- /              대시보드 화면 (index.html)
- /api/activity  활동 기록의 마지막 줄들 (JSON 배열)
- /font.ttf, /characters.png  게임 에셋을 그대로 보여 준다
"""
import json
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
LOG_FILE = ROOT / ".claude" / "activity" / "activity.jsonl"
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


class Handler(SimpleHTTPRequestHandler):
    def do_GET(self):
        path = self.path.split("?")[0]
        if path == "/api/activity":
            self.send(json.dumps(read_tail(), ensure_ascii=False).encode("utf-8"),
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
