# src/components/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 링크 |
|---|---|---|---|
| Layout.tsx | 109 | 셸: 사이드바, 모바일 드로어, 단축키 | [Layout.tsx.md](Layout.tsx.md) |
| HandwritingPad.tsx | 178 | 캔버스 스트로크 캡처 + Detexify 분류 | [HandwritingPad.tsx.md](HandwritingPad.tsx.md) |
| SymbolPanel.tsx | 139 | 기호 상세 패널 (별칭/도메인/예시/유사) | [SymbolPanel.tsx.md](SymbolPanel.tsx.md) |
| Sidebar.tsx | 53 | 좌측 네비 | (미선택 / deferred) |
| FormulaEditor.tsx | 84 | LaTeX 입력 에디터 | (미선택) |
| TokenList.tsx | 89 | 토큰 칩 리스트 | (미선택) |
| OcrDropzone.tsx | 98 | 이미지 드롭존 → `lib/ocr` | (미선택) |
| SpeechControls.tsx | 62 | TTS 재생/정지 | (미선택) |
| RenderView.tsx | 38 | MathJax 렌더 컨테이너 | (미선택) |
| ErrorBoundary.tsx | 32 | 표준 ErrorBoundary | (미선택) |
| icons.tsx | 40 | 인라인 SVG 아이콘 | (미선택) |

## 디렉토리 구조 / Structure Summary

`components/` 는 두 부류로 갈립니다:
1. **셸/네비게이션** — `Layout`, `Sidebar`, `ErrorBoundary` — 라우트 어느 곳에서나 공유.
2. **분석·검색 위젯** — `FormulaEditor`, `OcrDropzone`, `RenderView`, `SpeechControls`, `TokenList`, `SymbolPanel`, `HandwritingPad` — `AnalyzePage`/`SearchPage` 에서 조립됩니다.

별도의 hooks 디렉토리는 비어 있고, 각 컴포넌트는 `lib/`(부수효과 없는 모듈)나 `store/useAppStore`(Zustand 선택자)에 직접 의존합니다.

## 주요 의존 관계 / Key Dependencies

- `Layout` → `Sidebar`, `react-router-dom` `<Outlet/>`
- `HandwritingPad` → `lib/detexify.classifyStrokes`
- `SymbolPanel` → `ipc/client` (favorites add) + `store/useAppStore` (similar 클릭 시 토큰 선택)
- 다른 위젯은 부모 페이지가 props로 데이터를 전달.

## 위험 신호 / Risk Signals

1. `Layout` 단축키 핸들러가 input focus를 `document.querySelector("input[type='text']")` 로 추측 — UI 변경에 취약.
2. `HandwritingPad` 가 Canvas DPI 스케일링을 `useEffect` 1회만 — 윈도우 리사이즈/줌 시 좌표가 어긋날 가능성.
3. `SymbolPanel` 의 즐겨찾기 추가가 낙관적 UI 없이 fire-and-forget — 실패가 사용자에게 보이지 않음.
