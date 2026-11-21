"""server.py의 경로 가리기·오늘 요약 시험. 실행: python -I tools/dashboard/test_server.py
가짜 이름(홍길동)으로 시험하므로 진짜 이름은 어디에도 적지 않는다."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import server  # noqa: E402

ROOT = r"D:\홍길동\미쓰 게임"


def mask(s):
    return server.mask_text(s, ROOT)


class MaskTest(unittest.TestCase):
    def test_inside_root_becomes_relative(self):
        self.assertEqual(mask(r"D:\홍길동\미쓰 게임\game\src\main.rs"), r"game\src\main.rs")
        self.assertEqual(mask("D:/홍길동/미쓰 게임/game/src/main.rs"), "game/src/main.rs")
        self.assertEqual(mask("/d/홍길동/미쓰 게임/docs/a.md"), "docs/a.md")
        self.assertEqual(mask(r"d:\홍길동\미쓰 게임"), ".")

    def test_outside_root(self):
        self.assertEqual(mask(r"C:\Users\SDS\.claude\x.json"), "(프로젝트 밖)")
        self.assertEqual(mask(r"D:\ClaudeTemp\scratch\a.png"), "(프로젝트 밖)")
        self.assertEqual(mask("/c/Users/SDS/AppData/python.exe"), "(프로젝트 밖)")

    def test_inside_command(self):
        cmd = r'cd "D:/홍길동/미쓰 게임/tools/dashboard" && ls > C:\Temp\o.txt'
        self.assertEqual(mask(cmd), 'cd "tools/dashboard" && ls > (프로젝트 밖)')

    def test_truncated_path_leaves_no_name(self):
        self.assertNotIn("홍길동", mask('cd "D:/홍길동/미쓰 게'))
        self.assertNotIn("홍길동", mask("fragment 홍길동 left"))

    def test_plain_text_untouched(self):
        self.assertEqual(mask("cargo test --release 10/20"), "cargo test --release 10/20")
        self.assertEqual(mask("src/logic/game.rs"), "src/logic/game.rs")

    def test_clean_record_drops_private_fields(self):
        r = {"ts": 1.0, "event": "PreToolUse", "session": "s", "agent_type": "kevin", "tool": "Read",
             "input": {"file_path": "D:/홍길동/미쓰 게임/a.rs"},
             "transcript": r"C:\Users\SDS\x.jsonl", "keys": ["a"]}
        out = server.clean_record(r, ROOT)
        self.assertEqual(out["input"]["file_path"], "a.rs")
        self.assertNotIn("transcript", out)
        self.assertNotIn("session", out)


class TodayTest(unittest.TestCase):
    def rec(self, ts, agent, tool, fp=None, event="PreToolUse"):
        return {"ts": ts, "event": event, "agent_type": agent, "tool": tool,
                "input": {"file_path": fp} if fp else {}}

    def test_summary(self):
        inside = r"D:\홍길동\미쓰 게임\a.rs"
        recs = [
            self.rec(5, "ellie", "Read", inside),
            self.rec(6, "ellie", "Read", inside),
            self.rec(6, "ellie", "Read", inside, event="PostToolUse"),
            self.rec(7, "ellie", "Edit", r"C:\Users\SDS\밖.txt"),
            self.rec(8, "ellie", "Bash"),
            self.rec(9, None, "Agent"),
            self.rec(1, "bob", "Read", "x"),   # 어제
            self.rec(99, "bob", "Read", "x"),  # 내일
        ]
        s = server.summarize_today(recs, 2, 50, ROOT)
        self.assertEqual(set(s), {"ellie", "main"})
        e = s["ellie"]
        self.assertEqual((e["first"], e["last"], e["lines"], e["calls"]), (5, 8, 5, 4))
        self.assertEqual((e["read"], e["edit"], e["run"]), (2, 1, 1))
        self.assertEqual(e["top_files"], [{"path": "a.rs", "count": 2}])  # 프로젝트 밖 파일은 뺀다


if __name__ == "__main__":
    unittest.main()
