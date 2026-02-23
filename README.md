# rink

tmux 세션 대시보드. 여러 tmux 세션을 한눈에 보고, 빠르게 전환하고, 관리할 수 있습니다.

zellij를 외부 프레임으로 사용하여 왼쪽에 세션 목록 + 프리뷰, 오른쪽에 실제 tmux 터미널을 보여줍니다.

> macOS 전용. [nacyot/muxdash](https://github.com/nacyot/muxdash)에서 영감을 받았습니다.

## 설치

```bash
curl -fsSL https://raw.githubusercontent.com/seapy/rink/master/scripts/install.sh | bash
```

GitHub Releases에서 빌드된 바이너리를 다운로드합니다. Apple Silicon, Intel Mac 모두 지원.

tmux와 zellij는 첫 실행 시 Homebrew로 자동 설치됩니다.

PATH 설정이 필요할 수 있습니다:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## 실행

```bash
rink
```

좌 35%에 세션 대시보드, 우 65%에 tmux 터미널이 열립니다. 종료 후 다시 `rink`을 실행하면 이전 상태로 돌아갑니다.

zellij 없이 대시보드만 쓰려면:

```bash
rink --standalone
```

## 키바인딩

| 키 | 동작 |
|----|------|
| `↑`/`↓` 또는 `k`/`j` | 선택 이동 |
| `Enter` | 세션 전환 / 카테고리 포커스 |
| `/` | 검색 |
| `Tab`/`←`/`→` | 카테고리 접기/펼치기 |
| `c` | 세션 생성 |
| `x` | 세션/카테고리 삭제 |
| `R` | 세션 이름변경 |
| `C` | 카테고리 이름변경 (하위 세션 일괄) |
| `s` | 정렬 모드 순환 |
| `J`/`K` | 세션 순서 변경 (Custom 정렬) |
| `r` | 새로고침 |
| `?` | 도움말 |
| `Esc` | 취소 / 포커스 해제 |
| `Ctrl+x` | 종료 |

## 기능

### 프리뷰

선택한 세션의 터미널 출력이 하단에 실시간으로 표시됩니다. 세션을 전환하지 않고도 각 세션에서 무슨 일이 일어나고 있는지 확인할 수 있습니다.

### 카테고리

세션 이름에 구분자(`-`)가 있으면 자동으로 그룹핑됩니다:

```
work-api        → work 그룹
work-frontend   → work 그룹
personal-blog   → personal 그룹
scratch         → General 그룹
```

- `c`로 생성 시 현재 카테고리 prefix가 자동 채워짐
- `C`로 카테고리 이름을 바꾸면 하위 세션이 모두 일괄 rename

### 정렬

`s`키로 순환:

- **Name** - 알파벳순
- **Recent** - 최근 사용순
- **Windows** - 윈도우 수 많은 순
- **Custom** - `J`/`K`로 직접 순서 지정 (영속 저장)

### Claude Code 상태 표시

tmux 세션에서 Claude Code가 실행 중이면 상태 아이콘이 세션 옆에 표시됩니다:

```
● my-session *    ← 작업 중 (노란색)
○ other-session ? ← 사용자 입력 대기 (시안)
○ done-session +  ← 완료 (초록)
```

Hook 설치:

```bash
rink hook-install
```

`~/.claude/settings.json`에 자동으로 hook이 추가됩니다. Claude Code를 재시작하면 활성화됩니다.

## 설정

```bash
rink init
```

`~/.config/rink/config.toml`에 설정 파일이 생성됩니다:

```toml
separator = "-"           # 카테고리 구분자
refresh_interval_ms = 2000 # 새로고침 주기
default_sort = "name"      # name, recent, windows, custom
```
