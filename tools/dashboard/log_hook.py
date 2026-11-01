"""Claude Code 훅에서 불린다. 표준 입력의 훅 JSON을 짧게 줄여 활동 기록 파일에 한 줄씩 쌓는다.

기록 파일: .claude/activity/activity.jsonl (git에 올리지 않음)
대시보드(server.py)가 이 파일을 읽어 화면에 보여 준다.
"""
import json
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LOG_DIR = ROOT / ".claude" / "activity"
LOG_FILE = LOG_DIR / "activity.jsonl"
# 기록 파일이 이보다 커지면 뒤쪽 절반만 남긴다.
MAX_BYTES = 2_000_000


def short(text, limit=140):
    text = " ".join(str(text).split())
    return text if len(text) <= limit else text[: limit - 1] + "…"


def summarize_input(tool_input):
    """도구 입력에서 화면에 보여 줄 만한 것만 고른다. 파일 내용 같은 큰 값은 버린다."""
    if not isinstance(tool_input, dict):
        return {}
    keep = {}
    for key in ("file_path", "path", "pattern", "command", "description",
                "subagent_type", "url", "query", "action", "to", "skill"):
        if key in tool_input and tool_input[key] not in (None, ""):
            keep[key] = short(tool_input[key])
    return keep


def trim_if_big():
    try:
        if LOG_FILE.stat().st_size > MAX_BYTES:
            lines = LOG_FILE.read_text(encoding="utf-8").splitlines()
            LOG_FILE.write_text("\n".join(lines[len(lines) // 2:]) + "\n", encoding="utf-8")
    except FileNotFoundError:
        pass


def main():
    try:
        data = json.loads(sys.stdin.buffer.read().decode("utf-8") or "{}")
    except (ValueError, UnicodeDecodeError):
        return
    record = {
        "ts": time.time(),
        "event": data.get("hook_event_name"),
        "session": data.get("session_id"),
        "agent_id": data.get("agent_id"),
        "agent_type": data.get("agent_type"),
        "tool": data.get("tool_name"),
        "input": summarize_input(data.get("tool_input")),
        "transcript": data.get("transcript_path"),
        # 훅 입력에 어떤 필드가 오는지 확인하려고 키 이름만 남긴다.
        "keys": sorted(data.keys()),
    }
    if data.get("hook_event_name") == "PostToolUseFailure":
        record["error"] = short(data.get("error", ""), 200)
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    trim_if_big()
    with LOG_FILE.open("a", encoding="utf-8") as f:
        f.write(json.dumps(record, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
