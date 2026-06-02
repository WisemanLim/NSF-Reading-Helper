# src/pages/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List

| 파일 / File | LOC | 역할 / Role | 리뷰 링크 |
|---|---|---|---|
| HomePage.tsx | 51 | 랜딩 + 2개 진입 카드 | [HomePage.tsx.md](HomePage.tsx.md) |
| AnalyzePage.tsx | 103 | 메인 분석 화면 (입력→토큰→TTS) | [AnalyzePage.tsx.md](AnalyzePage.tsx.md) |
| SearchPage.tsx | 103 | 기호 사전 검색 + 손그림 패드 | [SearchPage.tsx.md](SearchPage.tsx.md) |
| LibraryPage.tsx | 74 | 최근 분석 + 즐겨찾기 | [LibraryPage.tsx.md](LibraryPage.tsx.md) |
| SettingsPage.tsx | 232 | 설정 + 업데이트 + 팩 import/export | [SettingsPage.tsx.md](SettingsPage.tsx.md) |

## 디렉토리 구조 / Structure Summary

각 페이지는 `Layout` 의 `<Outlet/>` 안에 렌더되며, **Zustand 선택자**를 통해 글로벌 상태를 구독합니다. 모든 백엔드 호출은 `ipc/client.ts` 로 추상화되어 Tauri/Web 동일 코드.

흐름 / Flow:
- `HomePage` 는 라우터 링크만 제공.
- `AnalyzePage` 가 가장 두꺼움: 입력→백엔드 analyze→koSpeak 폴리시→토큰 렌더→TTS.
- `SearchPage` 는 200ms 디바운스로 `ipc.symbolSearch` 호출.
- `LibraryPage` 는 첫 마운트 시 두 IPC 호출.
- `SettingsPage` 는 `isTauri` 조건부 섹션(업데이트/팩).

## 주요 의존 관계 / Key Dependencies

- 공통 / Common: `store/useAppStore`, `ipc/client`, `react-router-dom`.
- `AnalyzePage` → `components/{FormulaEditor,OcrDropzone,RenderView,SpeechControls,SymbolPanel,TokenList}` + `lib/koSpeak`.
- `SearchPage` → `components/{SymbolPanel,HandwritingPad}`.
- `SettingsPage` → `lib/updater` + `@tauri-apps/plugin-dialog` (동적 import).

## 위험 신호 / Risk Signals

1. `AnalyzePage.handleAnalyze` 가 백엔드 실패 시 빈 결과를 만든 뒤 다시 `setResult` — 사용자에게 "오류" 문자열만 노출, 실제 원인 표시 안 됨(원인은 console.warn).
2. `SearchPage` 의 detail 조회가 무조건 마지막 클릭 결과를 덮어씀(경쟁 조건 보호 없음). 빠르게 두 번 클릭하면 첫 응답이 뒤늦게 도착해 덮어쓸 수 있음.
3. `SettingsPage.onImportPack` 가 multiple=false 인데도 picked가 객체일 수 있는 분기를 처리(과거 API와의 호환). 신규 Tauri 2 dialog는 string 반환이라 그 분기는 dead path 가능성.
