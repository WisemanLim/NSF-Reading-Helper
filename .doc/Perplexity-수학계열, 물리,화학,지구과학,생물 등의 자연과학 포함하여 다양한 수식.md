<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# 수학계열, 물리,화학,지구과학,생물 등의 자연과학 포함하여 다양한 수식이 존재합니다. 수식에는 다양한 기호가 존재를 합니다. 기호명칭에 대한 읽기와 뜻에 대한 설명을 도와주는 프로그램을 구현하려고 합니다. 먼저 오픈소스를 조사하고 비교 및 추천해 주세요.

가능합니다. 이 목적의 프로그램은 **완성형 단일 오픈소스**를 그대로 가져다 쓰기보다, “수식 렌더링 + 의미 해석/음성화 + 기호 사전 + OCR/필기 인식”을 조합하는 방식이 가장 현실적입니다. 특히 웹 기반 구현이라면 MathJax/SRE 축이 가장 강력하고, 이미지·손글씨 입력까지 받으려면 pix2tex류를 보완적으로 붙이는 구성이 유력합니다.[^1_1][^1_2][^1_3][^1_4]

## 요구사항 분해

질문하신 프로그램은 보통 다음 4개 기능으로 나뉩니다. 수식을 화면에 정확히 표시하는 엔진, 기호를 사람이 읽는 문장으로 바꾸는 엔진, 각 기호의 명칭·뜻을 설명하는 지식베이스, 그리고 이미지·필기 입력을 LaTeX/MathML로 바꾸는 인식 모듈이 필요합니다.[^1_2][^1_3][^1_1]

자연과학 전반을 다루려면 단순한 “수학 기호 표”만으로는 부족합니다. 예를 들어 그리스 문자, 연산자, 논리기호, 화학식 표기, 단위 표기, 지구과학 기호, 생물학·통계학에서 쓰는 약어와 첨자 관례까지 확장해야 하므로, 코어 엔진과 도메인 사전을 분리한 아키텍처가 적합합니다.[^1_5][^1_6]

## 주요 오픈소스

| 오픈소스 | 핵심 역할 | 장점 | 한계 | 추천도 |
| :-- | :-- | :-- | :-- | :-- |
| MathJax | LaTeX/MathML/AsciiMath 렌더링 및 접근성 기반 웹 표시 [^1_1][^1_5] | 웹 표준 친화적이고 보조공학 지원이 강함, SRE와 연결 가능 [^1_1][^1_4] | 기호 “뜻 설명 DB” 자체는 약함, 앱 레벨 확장 필요 [^1_4] | 매우 높음 |
| Speech Rule Engine (SRE) | 수식을 읽기 문장으로 변환, Mathspeak/Clearspeak/점자 지원 [^1_3][^1_7] | 수식의 **읽기** 기능에 가장 직접적, 로컬/브라우저/Node 사용 가능 [^1_3] | 기호별 교육용 설명 UI는 직접 만들어야 함 [^1_3][^1_8] | 매우 높음 |
| KaTeX | 빠른 웹 수식 렌더링 [^1_9] | 속도가 빠르고 프론트엔드 통합이 쉬움 [^1_9] | 접근성·음성화 측면은 MathJax+SRE 조합보다 약함 [^1_9][^1_4] | 중간 |
| AsciiMath | 간단한 수식 입력 문법 [^1_10] | 초중등·입문자 입력에 유리 [^1_10][^1_5] | 복잡한 과학 수식 전체를 포괄하기엔 한계 [^1_10] | 보조용 |
| LaTeX-OCR / pix2tex | 이미지 속 수식을 LaTeX로 변환 [^1_2][^1_11] | 스캔·캡처 수식 입력을 지원하기 좋음 [^1_2] | 읽기/뜻 설명 기능은 별도 구현 필요 [^1_2][^1_11] | 보조용 |
| Pix2Text | 문서·수식·텍스트 혼합 OCR [^1_2][^1_12] | 수식 외 일반 텍스트와 함께 추출 가능 [^1_2] | 기호명 설명 엔진은 아님 [^1_12] | 보조용 |
| Detexify 계열 | 손으로 그린 기호를 LaTeX 기호 후보로 찾음 [^1_13][^1_14] | “기호 이름은 모르지만 모양은 안다”는 사용성에 매우 적합 [^1_13] | 수식 전체 해석보다는 기호 탐색용 [^1_13][^1_14] | 보조용 |
| LaTeXML | LaTeX를 의미 있는 HTML/기타 포맷으로 변환 [^1_6][^1_15] | 문서 단위 의미 구조 확보에 유리 [^1_6] | 실시간 인터랙티브 앱의 1차 엔진으로는 다소 무거움 [^1_15][^1_6] | 문서처리용 |

## 비교 포인트

핵심은 “보여주기”보다 “읽어주기와 설명하기”입니다. 이 기준에서는 KaTeX보다 MathJax가 유리한데, MathJax는 접근성 컴포넌트와 SRE 연계를 공식적으로 제공하며, 수식을 음성 문자열로 바꾸는 경로가 명확합니다.[^1_4][^1_1]

SRE는 이 프로젝트의 중심 후보입니다. SRE는 MathML 및 MathJax DOM을 기반으로 수식을 음성 문자열로 바꾸고, Mathspeak와 Clearspeak 같은 읽기 규칙을 제공하므로 “$\int$”를 단순히 기호 코드가 아니라 문맥 있는 읽기 형태로 제시할 수 있습니다.[^1_3][^1_8]

입력 편의성 측면에서는 OCR과 손그림 인식이 중요합니다. pix2tex/LaTeX-OCR는 이미지 속 수식을 LaTeX로 바꾸는 데 유용하고, Detexify 계열은 사용자가 기호 모양만 알고 이름을 모를 때 검색 경험을 크게 개선합니다.[^1_14][^1_13][^1_2]

## 추천 아키텍처

가장 추천하는 조합은 **MathJax + SRE + 자체 기호 사전 + 선택적 OCR/필기 인식**입니다. MathJax가 입력된 LaTeX/MathML/AsciiMath를 렌더링하고, SRE가 이를 읽기 문장으로 변환하며, 앱은 별도의 사전 테이블에서 각 토큰에 대해 “명칭”, “한국어 읽기”, “의미”, “분야별 예시”를 붙이는 구조가 좋습니다.[^1_1][^1_3][^1_4]

구체적으로는 다음 구조가 현실적입니다.

- 입력층: LaTeX, MathML, AsciiMath, 이미지 OCR, 손그림 기호 검색[^1_10][^1_13][^1_2]
- 파싱/표시층: MathJax 우선, 필요 시 문서 처리용 LaTeXML 병행[^1_6][^1_1]
- 읽기층: SRE 기반 Clearspeak/Mathspeak[^1_16][^1_3]
- 설명층: 자체 구축 기호 사전, 예를 들어 `symbol`, `latex`, `unicode`, `ko_name`, `ko_reading`, `meaning`, `domain`, `examples` 스키마
- 도메인 확장층: 화학식, 생물 통계, 물리 단위, 천문·지구과학 기호 룰셋

이때 기호 사전은 외부 오픈소스를 그대로 찾기보다, Unicode 수학 기호 목록과 KaTeX/LaTeX 지원 심볼 테이블을 바탕으로 내부 DB를 만드는 편이 안정적입니다. 공개된 UnicodeMath 계열 자료나 unicode-math 테이블은 초기 시드 데이터로 활용 가치가 있습니다.[^1_17][^1_9][^1_18]

## 최종 추천

1. **1순위: MathJax + SRE 기반 웹앱**

- 이유: 접근성, 읽기 기능, 웹 통합성, 확장성의 균형이 가장 좋습니다.[^1_3][^1_4][^1_1]
- 적합한 경우: “기호 읽기 + 뜻 설명 + 웹 UI”가 핵심인 경우.[^1_4][^1_3]

2. **2순위: MathJax + SRE + pix2tex**

- 이유: 사용자가 논문 캡처나 이미지에서 수식을 넣는 시나리오까지 커버할 수 있습니다.[^1_11][^1_2]
- 적합한 경우: 교육용·문서 분석용·연구지원용 도구를 만들 때.[^1_2]

3. **3순위: KaTeX + 자체 사전**

- 이유: 렌더링은 빠르지만, 읽기/접근성 중심 제품으로 가면 결국 추가 보완이 많이 필요합니다.[^1_9][^1_4]
- 적합한 경우: 아주 가벼운 프론트엔드 위젯이나 오프라인 렌더링 중심 서비스.[^1_9]

현 시점에서 귀하의 목표인 “기호 명칭 읽기와 뜻 설명”에는 **MathJax + SRE를 코어로 채택**하고, 여기에 **도메인별 기호 사전**을 별도로 얹는 방식을 가장 추천합니다. OCR은 필수가 아니라 옵션으로 뒤에 붙이는 것이 개발 리스크와 정확도 측면에서 좋습니다.[^1_2][^1_3][^1_4]

다음 단계로 진행하면 좋습니다.

- 1단계: MVP 범위를 수학/물리 기호 중심으로 제한
- 2단계: 입력 포맷을 LaTeX + 직접 기호 클릭으로 시작
- 3단계: SRE 기반 읽기 문자열 생성
- 4단계: 한국어 기호 사전과 분야별 설명 추가
- 5단계: 이후 OCR, 손그림 인식, 화학식 확장 도입[^1_3][^1_4][^1_2]

원하시면 다음 답변에서 바로 이어서
**“후보 오픈소스별 라이선스/활성도/기술스택/장단점 표”**와 함께
**추천 MVP 아키텍처 및 초기 DB 스키마**까지 구체적으로 설계해 드리겠습니다.
<span style="display:none">[^1_19][^1_20][^1_21][^1_22][^1_23][^1_24][^1_25][^1_26][^1_27][^1_28][^1_29][^1_30][^1_31][^1_32][^1_33][^1_34][^1_35][^1_36][^1_37][^1_38][^1_39][^1_40][^1_41][^1_42][^1_43][^1_44][^1_45]</span>

<div align="center">⁂</div>

[^1_1]: https://www.mathjax.org

[^1_2]: https://github.com/topics/math-ocr

[^1_3]: https://speechruleengine.org

[^1_4]: https://docs.mathjax.org/en/v4.0/web/components/accessibility.html

[^1_5]: https://accessibility.ecampusontario.ca/accessibility/best-practices/equations/

[^1_6]: https://www.ams.org/accessibility-tools

[^1_7]: https://github.com/Speech-Rule-Engine

[^1_8]: https://github.com/speech-rule-engine/speech-rule-engine

[^1_9]: https://www.npmjs.com/package/katex

[^1_10]: https://asciimath.org

[^1_11]: https://www.reddit.com/r/ObsidianMD/comments/tjcwaz/latex_math_ocr_tool/

[^1_12]: https://groups.google.com/g/tesseract-ocr/c/3_Hoy3xkW18

[^1_13]: https://detexify.kirelabs.org

[^1_14]: https://dev.to/yuhuishishishi/detexify-in-your-browser-43k

[^1_15]: https://tuoselevate.blogspot.com/2022/05/creating-accessible-maths-notes-with.html

[^1_16]: https://runestone.academy/ns/books/published/pretextguide/install-mathjax.html

[^1_17]: https://github.com/jipsen/UnicodeMath

[^1_18]: https://github.com/latex3/unicode-math/blob/master/unicode-math-table.tex

[^1_19]: https://www.mathcha.io

[^1_20]: https://math.typeit.org

[^1_21]: https://developer.android.com/reference/kotlin/android/icu/number/ScientificNotation

[^1_22]: https://www.reddit.com/r/learnmath/comments/10a7g8k/free_website_for_creating_copying_and_pasting/

[^1_23]: https://dl.acm.org/doi/full/10.1145/3688671.3688771

[^1_24]: https://www.section508.gov/create/accessible-equations-formulas/

[^1_25]: https://mathpix.com/image-to-latex

[^1_26]: https://www.khanacademy.org/v/scientific-notation

[^1_27]: https://stackoverflow.com/questions/3570220/ocr-lib-for-math-formulas

[^1_28]: https://www.obsidianstats.com/plugins/latex-ocr

[^1_29]: https://github.com/latex3/tagging-project/discussions/816

[^1_30]: https://www.facnewsletter.nsm.uh.edu/issues/september-2023/articles/improving-canvas/VLC-Accessible-Math-Guide.pdf

[^1_31]: https://www.youtube.com/watch?v=wufqq72hhcM

[^1_32]: https://dl.acm.org/doi/pdf/10.1145/3315002.3317567

[^1_33]: https://asarantsev.github.io/WebArchive/accessdetail.html

[^1_34]: https://mathjax.github.io/MathJax-a11y/

[^1_35]: https://latex.org/forum/viewtopic.php?t=36179

[^1_36]: https://gist.github.com/jaantollander/b49132a36b035cdfe05345dd0c7edf0f

[^1_37]: https://gist.github.com/taliesinb/4c2700c543c6e06a47a8a3cc85bb9772

[^1_38]: https://github.com/topics/math-symbols

[^1_39]: https://gist.github.com/mk1564/starred

[^1_40]: https://yingtongli.me/blog/2022/09/24/katex-custom-fonts.html

[^1_41]: https://github.com/gatapia/unicode-math-vscode/blob/master/mathsymbols.py

[^1_42]: https://www.reddit.com/r/opensource/comments/11kf56/detexify_automatic_latex_recognition_draw_a/

[^1_43]: https://stackoverflow.com/questions/36666141/greek-fonts-for-katex

[^1_44]: https://github.com/mvoidex/UnicodeMath/blob/master/mathsymbols.py

[^1_45]: https://www.drawtex.io/about


---

# “후보 오픈소스별 라이선스/활성도/기술스택/장단점 표”와 함께 추천 MVP 아키텍처 및 초기 DB 스키마까지 구체적으로 설계해 주세요.

추천 MVP는 **MathJax + Speech Rule Engine(SRE) + PostgreSQL 기반 기호 사전**을 코어로 두고, 2단계에서 OCR/필기 인식을 붙이는 구조입니다. 이 조합은 웹 렌더링, 수식 읽기, 접근성, 향후 과학 분야 확장성의 균형이 가장 좋습니다.[^2_1][^2_2][^2_3]

## 후보 비교표

아래 표는 “기호 읽기 + 뜻 설명 프로그램” 관점에서 바로 검토 가능한 후보들만 추렸습니다. 라이선스는 상용 포함 배포 가능성에 큰 영향을 주므로, MVP 단계에서는 Apache-2.0, MIT 같은 **permissive** 계열을 우선 추천합니다.[^2_4][^2_5][^2_6][^2_7]


| 오픈소스 | 주 역할 | 라이선스 | 활성도/현황 | 기술스택 | 장점 | 한계 |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| MathJax | LaTeX/MathML/AsciiMath 렌더링, 접근성 지원 [^2_2][^2_3] | Apache-2.0 [^2_4][^2_5][^2_8] | npm 패키지 제공, GitHub/NPM 생태계가 크고 유지 중 [^2_9][^2_10] | JavaScript/TypeScript, 브라우저/Node [^2_9][^2_8] | 웹 렌더링 표준에 가깝고 접근성 확장이 강함, SRE와 결합 용이 [^2_3][^2_11] | 기호 뜻 설명용 지식베이스는 내장되어 있지 않음 [^2_3] |
| Speech Rule Engine | 수식을 읽기 문장으로 변환, Mathspeak/Clearspeak 등 제공 [^2_1][^2_12] | Apache-2.0, 관련 LaTeX-to-Speech도 Apache-2.0 [^2_13][^2_6] | TypeScript 기반으로 발전 중이며 브라우저·Node·CLI 사용 가능 [^2_1] | TypeScript/JavaScript [^2_1] | 이 프로젝트의 핵심인 “수식 읽기”에 가장 직접적, 규칙 기반 확장 가능 [^2_1] | 한국어 읽기 규칙과 교육용 설명 UI는 별도 개발 필요 [^2_1] |
| KaTeX | 고속 수식 렌더링 [^2_14] | MIT, npm 패키지 제공 [^2_15][^2_14][^2_16] | 널리 사용되며 Node/브라우저 통합이 쉬움 [^2_15][^2_14] | JavaScript [^2_14] | 렌더링 속도가 빠르고 프론트엔드 위젯화가 쉬움 [^2_14] | 접근성·읽기 기능은 MathJax+SRE 조합보다 약함 [^2_3][^2_14] |
| LaTeX-OCR / pix2tex | 이미지 속 수식을 LaTeX로 변환 [^2_17][^2_18] | MIT [^2_18] | GitHub star 규모가 크고 OCR 대안으로 널리 언급됨 [^2_18] | Python, ViT 기반 OCR 계열 [^2_17][^2_18] | 스캔 이미지·논문 캡처 입력을 지원하기 좋음 [^2_17] | 읽기/뜻 설명 기능은 전혀 별도 구현 필요 [^2_17][^2_18] |
| Pix2Text | 문서·텍스트·표·수식 혼합 OCR [^2_19][^2_20] | MIT [^2_20][^2_21] | 오픈소스 대안으로 성장 중이며 수식 포함 문서 처리 범위가 넓음 [^2_20][^2_21] | Python [^2_19][^2_20] | PDF/문서 전체에서 수식과 일반 텍스트를 함께 뽑기 좋음 [^2_20] | 인터랙티브 수식 읽기 엔진은 아님 [^2_20] |
| Detexify | 손그림 기호를 LaTeX 후보로 검색 [^2_22][^2_23] | 소스는 MIT, 학습 데이터는 ODbL [^2_7][^2_24] | 오래된 프로젝트지만 여전히 유용한 사용성 포인트 제공 [^2_22][^2_23] | 웹 프론트/백엔드, 필기 기호 인식 [^2_22][^2_25] | “모양은 아는데 이름 모르는 기호” 탐색 UX가 매우 좋음 [^2_22] | 전체 수식 해석 엔진이 아니라 기호 탐색 보조 도구에 가까움 [^2_22][^2_25] |
| UnicodeMath / unicode-math 자료 | 기호 테이블·매핑 데이터의 초기 시드 [^2_26][^2_27] | LPPL 계열 자료 존재 [^2_28][^2_27] | 수학 기호 커버리지가 넓음 [^2_27] | TeX/기호 테이블 [^2_27] | 자체 기호 사전 구축의 기초 데이터로 활용 가치가 큼 [^2_27][^2_29] | 서비스용 지식베이스로 바로 쓰기엔 설명 필드와 도메인 메타데이터가 부족함 [^2_27] |

## 추천 조합

MVP 기준 1순위는 **MathJax + SRE + 자체 기호 사전 DB**입니다. MathJax가 수식을 안정적으로 렌더링하고, SRE가 수식 전체 또는 부분식을 자연스러운 읽기 문자열로 바꾸며, DB가 기호별 명칭·뜻·분야별 설명을 보강하는 역할을 맡습니다.[^2_2][^2_3][^2_1]

2단계 확장으로는 **pix2tex 또는 Pix2Text**, 3단계 확장으로는 **Detexify 계열 입력 UX**를 붙이는 것이 좋습니다. 이렇게 하면 입력 방식이 “직접 입력 → 이미지 OCR → 손그림 기호 검색” 순으로 자연스럽게 넓어집니다.[^2_17][^2_20][^2_22]

실무 관점에서 KaTeX는 “빠른 렌더링 위젯”으로는 좋지만, 현재 목표가 교육형·설명형 프로그램이라면 코어 엔진으로는 MathJax가 더 적합합니다. 특히 접근성 컴포넌트와 assistive MathML, SRE 연계 문서가 이미 존재한다는 점이 중요합니다.[^2_11][^2_3]

## MVP 아키텍처

권장 아키텍처는 웹앱 기준으로 다음과 같습니다. 프론트엔드는 수식 입력과 기호 탐색 UI를 제공하고, 백엔드는 파싱·읽기 생성·기호 메타데이터 조회를 담당하는 **API 중심 구조**가 좋습니다.[^2_1][^2_2]

### 구성 요소

- Frontend: React 또는 Next.js, 수식 입력기, 기호 패널, 결과 뷰어.
- Render Engine: MathJax, 필요 시 부분 토큰 클릭 이벤트와 assistive 정보 추출.[^2_3][^2_2]
- Speech Engine: SRE, 수식 전체 읽기와 부분식 읽기 생성.[^2_13][^2_1]
- API Server: Node.js/TypeScript 권장, SRE와 MathJax 서버사이드 보조 처리.[^2_8][^2_1]
- DB: PostgreSQL, 기호 사전·도메인 분류·예문·동의어 저장.
- Search: PostgreSQL Full Text Search 또는 pg_trgm, 기호명·별칭·LaTeX 명령 검색.
- Optional OCR Worker: Python FastAPI + pix2tex/Pix2Text.[^2_20][^2_17]
- Optional Handwriting Service: Detexify 계열 모델 또는 외부 모듈.[^2_22][^2_7]


### 요청 흐름

1. 사용자가 LaTeX/MathML/AsciiMath/문자 기호를 입력합니다.[^2_2]
2. 서버가 입력을 정규화하여 내부 표현식으로 변환합니다.[^2_1][^2_2]
3. MathJax가 렌더링용 구조를 만들고, SRE가 읽기 문자열을 생성합니다.[^2_3][^2_1]
4. 토큰 단위로 추출된 기호를 DB에서 조회해 “명칭/읽기/뜻/분야 설명”을 결합합니다.
5. 프론트엔드는 “전체 읽기”, “기호별 설명”, “분야별 해설”, “유사 기호 비교”를 동시에 표시합니다.

### 화면 단위 MVP

- 입력 패널: LaTeX, Unicode 수학기호, 간단 텍스트 입력.
- 렌더링 패널: MathJax 결과.
- 읽기 패널: 전체 수식 읽기, 단계별 읽기.
- 기호 설명 패널: 클릭한 기호의 명칭, 한글 읽기, 의미, 사용 예.
- 검색 패널: “적분”, “∫”, “\int”, “integral” 모두 검색 가능.
- 비교 패널: 비슷한 기호, 예를 들어 $\in$, $\ni$, $\subset$, $\subseteq$.


## 서비스 내부 모델

MVP에서 가장 중요한 개념은 “수식 전체”와 “기호 단위”를 분리하는 것입니다. 수식은 문맥을 가지므로 SRE가 읽기에 강하고, 기호는 사전형 설명이 필요하므로 DB가 강합니다. 따라서 **Expression 모델**과 **Symbol 모델**을 분리해야 합니다.[^2_3][^2_1]

권장 내부 파이프라인은 다음과 같습니다.

- Input Normalizer: LaTeX, MathML, AsciiMath, Unicode를 공통 포맷으로 정규화.[^2_2]
- Token Extractor: 명령, 식별자, 연산자, 관계기호, 구분자, 함수, 첨자/위첨자 등을 분해.
- Semantic Mapper: 토큰을 DB symbol 엔트리와 매핑.
- Speech Composer: 전체 수식 읽기, 단계별 읽기, 부분식 읽기 생성.[^2_1]
- Explanation Composer: “이 기호는 …이며, 주로 …에서 쓰인다” 식의 설명 템플릿 생성.


## 초기 DB 스키마

초기에는 너무 많은 정규화보다, 빠르게 기호 지식베이스를 쌓을 수 있도록 적당한 정규화가 좋습니다. PostgreSQL 기준으로 아래 7개 테이블이면 MVP를 충분히 시작할 수 있습니다.

### 1) domains

분야 분류입니다. 수학, 물리, 화학, 지구과학, 생물, 통계, 논리, 단위계 등을 담습니다.

```sql
create table domains (
  id bigserial primary key,
  code varchar(50) unique not null,
  name_ko varchar(100) not null,
  name_en varchar(100),
  description text,
  created_at timestamptz default now()
);
```

예시 데이터:

- `math-algebra`
- `math-calculus`
- `physics-general`
- `chemistry`
- `earth-science`
- `biology`
- `statistics`
- `logic`


### 2) symbols

핵심 테이블입니다. 하나의 기호 또는 명령 단위를 표현합니다.

```sql
create table symbols (
  id bigserial primary key,
  unicode_char varchar(16),
  latex_command varchar(128),
  mathml_tag varchar(128),
  canonical_name_ko varchar(200) not null,
  canonical_name_en varchar(200),
  reading_ko varchar(200),
  reading_en varchar(200),
  short_meaning_ko text not null,
  short_meaning_en text,
  symbol_type varchar(50) not null,
  difficulty_level smallint default 1,
  is_operator boolean default false,
  is_relation boolean default false,
  is_delimiter boolean default false,
  is_greek boolean default false,
  is_constant boolean default false,
  is_function boolean default false,
  notes text,
  created_at timestamptz default now(),
  updated_at timestamptz default now()
);
```

추천 `symbol_type` 값:

- operator
- relation
- delimiter
- greek-letter
- function
- arrow
- set-symbol
- logic-symbol
- chemistry-notation
- unit-symbol
- biology-notation


### 3) symbol_aliases

검색성과 현업 입력 다양성을 위해 필수입니다. 예를 들어 “시그마”, “Σ”, “sigma”, `\Sigma`를 모두 같은 심볼에 연결합니다.

```sql
create table symbol_aliases (
  id bigserial primary key,
  symbol_id bigint not null references symbols(id) on delete cascade,
  alias_text varchar(200) not null,
  alias_type varchar(50) not null,
  locale varchar(10) default 'ko',
  unique(symbol_id, alias_text, alias_type, locale)
);
```

추천 `alias_type`:

- ko-name
- en-name
- pronunciation
- latex
- unicode
- synonym
- school-term


### 4) symbol_domain_map

한 기호가 여러 분야에서 서로 다른 의미를 갖는 경우가 많으므로 다대다 매핑이 필요합니다.

```sql
create table symbol_domain_map (
  id bigserial primary key,
  symbol_id bigint not null references symbols(id) on delete cascade,
  domain_id bigint not null references domains(id) on delete cascade,
  priority smallint default 1,
  meaning_in_domain_ko text,
  meaning_in_domain_en text,
  example_usage text,
  unique(symbol_id, domain_id)
);
```

예를 들어 `μ`는 수학에서는 measure 또는 parameter 문맥, 물리에서는 micro 또는 reduced mass, 생물·화학 문맥에서는 micro-prefix 등으로 달라질 수 있습니다.

### 5) expressions

사용자가 입력한 원본 수식과 정규화 결과를 저장합니다. 추후 학습 로그, 추천, 검색 고도화에 매우 유용합니다.

```sql
create table expressions (
  id bigserial primary key,
  input_format varchar(20) not null,
  raw_input text not null,
  normalized_latex text,
  normalized_mathml text,
  rendered_ok boolean default false,
  speech_text_ko text,
  speech_text_en text,
  created_at timestamptz default now()
);
```

추천 `input_format`:

- latex
- mathml
- asciimath
- unicode
- image-ocr
- handwriting


### 6) expression_symbols

수식과 기호의 연결입니다. 토큰 위치를 저장해 UI에서 “클릭한 위치의 기호 설명”을 가능하게 합니다.

```sql
create table expression_symbols (
  id bigserial primary key,
  expression_id bigint not null references expressions(id) on delete cascade,
  symbol_id bigint not null references symbols(id),
  token_text varchar(200) not null,
  token_index integer not null,
  start_pos integer,
  end_pos integer,
  ast_path varchar(255),
  role varchar(50),
  unique(expression_id, token_index)
);
```

추천 `role`:

- identifier
- operator
- superscript
- subscript
- numerator
- denominator
- function-name
- bound-variable


### 7) symbol_examples

기호별 예문과 분야 예문을 저장합니다. 교육형 서비스에서는 체감 가치가 큽니다.

```sql
create table symbol_examples (
  id bigserial primary key,
  symbol_id bigint not null references symbols(id) on delete cascade,
  domain_id bigint references domains(id),
  example_latex text not null,
  example_reading_ko text,
  example_explanation_ko text,
  difficulty_level smallint default 1,
  created_at timestamptz default now()
);
```


## 인덱스 전략

검색 경험이 핵심이므로 초기에 아래 인덱스는 꼭 넣는 편이 좋습니다.

```sql
create index idx_symbols_unicode_char on symbols(unicode_char);
create index idx_symbols_latex_command on symbols(latex_command);
create index idx_symbols_symbol_type on symbols(symbol_type);
create index idx_aliases_alias_text on symbol_aliases(alias_text);
create index idx_expression_symbols_expression on expression_symbols(expression_id);
```

PostgreSQL에서 fuzzy 검색까지 염두에 두면 `pg_trgm`를 붙여 `alias_text`, `canonical_name_ko`, `latex_command`에 trigram index를 추가하는 것이 좋습니다. 그러면 사용자가 `\alph`, `sigmaa`, `인테그랄`처럼 오타를 내도 검색 보정이 가능합니다.

## 초기 데이터 예시

MVP 시드 데이터는 처음부터 전 분야 전체를 넣기보다, 학습 가치와 사용 빈도가 높은 코어 기호부터 시작해야 합니다. 추천 초기 범위는 그리스 문자, 기본 연산자, 비교/관계 기호, 집합 기호, 미적분 기호, 물리 상수/단위 기호, 화학 반응 화살표와 상태 표기 정도입니다.[^2_27][^2_29]

예시:

```sql
insert into symbols (
  unicode_char, latex_command, canonical_name_ko, canonical_name_en,
  reading_ko, reading_en, short_meaning_ko, symbol_type, is_operator
) values
('∫', '\int', '적분 기호', 'integral sign', '인테그럴', 'integral',
 '연속적인 변화량을 누적하여 전체 값을 구할 때 사용하는 기호', 'operator', true),
('Σ', '\Sigma', '대문자 시그마', 'capital sigma', '시그마', 'sigma',
 '합을 나타내거나 특정 문맥에서 행렬·집합을 나타낼 때 쓰는 기호', 'greek-letter', false),
('≤', '\leq', '작거나 같다', 'less than or equal to', '레크 또는 이하', 'less than or equal to',
 '왼쪽 값이 오른쪽 값보다 작거나 같음을 나타내는 관계 기호', 'relation', false);
```


## API 설계 초안

MVP는 REST로 충분합니다. 나중에 수식 편집기와 실시간 분석이 커지면 WebSocket을 붙이면 됩니다.

- `POST /api/expressions/analyze`
    - 입력: `rawInput`, `format`, `locale`
    - 출력: 렌더링용 HTML/MathML, 전체 읽기, 토큰 목록, 기호 설명 목록
- `GET /api/symbols/search?q=...`
    - 기호명, LaTeX, Unicode, 별칭 검색
- `GET /api/symbols/:id`
    - 단일 기호 상세
- `GET /api/symbols/:id/examples`
    - 분야별 예문
- `POST /api/ocr/formula`
    - 이미지 업로드 후 LaTeX 반환, 2단계 기능[^2_17][^2_20]

응답 예시 개념:

```json
{
  "normalizedLatex": "\\int_0^1 x^2 dx",
  "speechKo": "0부터 1까지 x 제곱을 x에 대하여 적분",
  "tokens": [
    {
      "token": "\\int",
      "symbolId": 101,
      "nameKo": "적분 기호",
      "readingKo": "인테그럴",
      "meaningKo": "연속적인 변화량을 누적하는 연산 기호"
    }
  ]
}
```


## MVP 구현 우선순위

개발 리스크를 줄이려면 아래 순서가 좋습니다.

1. **1차 MVP**

- LaTeX 입력
- MathJax 렌더링[^2_2]
- SRE 기반 전체 읽기 생성[^2_1]
- 기호 클릭 시 DB 설명 표시

2. **1.5차**

- Unicode 직접 입력, 예: `∑`, `√`, `≤`
- 기호 검색창, 별칭 검색
- 유사 기호 비교 UI

3. **2차**

- AsciiMath 지원[^2_30][^2_2]
- 이미지 OCR, pix2tex 또는 Pix2Text 연동[^2_20][^2_17]

4. **3차**

- 손그림 기호 검색, Detexify형 UX[^2_7][^2_22]
- 화학식/생물·지구과학 도메인 룰셋 강화


## 기술 선택 제안

귀하의 백그라운드를 고려하면 백엔드는 **Node.js + TypeScript + PostgreSQL**, OCR 워커는 **Python FastAPI**, 프론트는 **Next.js**가 가장 무난합니다. SRE가 JavaScript/TypeScript 친화적이므로 핵심 읽기 엔진을 Node 영역에 두는 편이 자연스럽고, OCR만 Python으로 분리하면 ML 의존성과 운영 배포를 깔끔하게 분리할 수 있습니다.[^2_8][^2_17][^2_1]

배포는 다음처럼 나누는 편이 좋습니다.

- Web/API: Docker + Kubernetes 또는 단일 VM
- OCR Worker: GPU 가능 노드 또는 비동기 작업 큐
- DB: PostgreSQL
- 캐시: Redis, 빈번한 기호 조회 및 최근 분석 결과 캐싱


## 설계 판단

이 프로젝트의 핵심 차별점은 렌더링이 아니라 **“설명 가능한 수식 해석”**입니다. 따라서 엔진 선택보다 더 중요한 것은 “수식 전체 읽기”와 “기호 단위 교육 설명”을 결합하는 데이터 모델이며, 그 점에서 자체 DB 스키마를 중심에 두는 설계가 맞습니다.[^2_3][^2_1]

가장 현실적인 시작점은 수학·물리 공통 기호 200~500개 정도를 먼저 구축하고, 사용자 입력 로그를 통해 화학·생물·지구과학 확장을 점진적으로 하는 방식입니다. unicode-math 계열 자료는 기호 목록의 시드로 활용하고, 설명 문안과 분야별 의미는 직접 큐레이션하는 것이 품질 면에서 훨씬 좋습니다.[^2_28][^2_27]

다음 단계로는 **ERD 수준의 상세 스키마**, **Spring Boot 대신 Node를 택할지 비교**, 또는 **바로 실행 가능한 PostgreSQL DDL + 샘플 데이터 100건 구조**까지 이어서 설계할 수 있습니다.
<span style="display:none">[^2_31][^2_32][^2_33][^2_34][^2_35][^2_36][^2_37][^2_38][^2_39][^2_40][^2_41][^2_42][^2_43][^2_44][^2_45][^2_46][^2_47][^2_48][^2_49]</span>

<div align="center">⁂</div>

[^2_1]: https://speechruleengine.org

[^2_2]: https://www.mathjax.org

[^2_3]: https://docs.mathjax.org/en/v4.0/web/components/accessibility.html

[^2_4]: https://github.com/mathjax/MathJax/blob/master/LICENSE

[^2_5]: https://cloudsmith.com/navigator/npm/mathjax-full

[^2_6]: https://github.com/mathjax/MathJax-a11y

[^2_7]: https://github.com/kirel/detexify/blob/master/MIT-LICENSE

[^2_8]: https://github.com/mathjax/MathJax-src/blob/master/package.json

[^2_9]: https://www.npmjs.com/package/mathjax

[^2_10]: https://cloudsmith.com/navigator/npm/mathjax

[^2_11]: https://github.com/mathjax/MathJax-docs/blob/master/basic/accessibility.rst

[^2_12]: https://github.com/speech-rule-engine/speech-rule-engine

[^2_13]: https://github.com/Speech-Rule-Engine/sre-latex

[^2_14]: https://katex.org/docs/node

[^2_15]: https://www.npmjs.com/package/katex

[^2_16]: https://www.jsdelivr.com/package/npm/ng-katex-2

[^2_17]: https://github.com/lukas-blecher/LaTeX-OCR

[^2_18]: https://repos.ecosyste.ms/hosts/GitHub/repositories/lukas-blecher%2FLaTeX-OCR

[^2_19]: https://github.com/breezedeus/Pix2Text

[^2_20]: https://sourceforge.net/projects/pix2text.mirror/

[^2_21]: https://awesome.ecosyste.ms/projects/github.com%2Fbreezedeus%2FPix2Text

[^2_22]: https://detexify.kirelabs.org

[^2_23]: https://formulae.brew.sh/cask/detexify

[^2_24]: https://github.com/kirel/detexify-data

[^2_25]: https://dev.to/yuhuishishishi/detexify-in-your-browser-43k

[^2_26]: https://github.com/jipsen/UnicodeMath

[^2_27]: https://ctan.org/pkg/unicode-math

[^2_28]: https://gist.github.com/jaantollander/b49132a36b035cdfe05345dd0c7edf0f

[^2_29]: https://github.com/LucasAschenbach/vscode-unicode-math

[^2_30]: https://asciimath.org

[^2_31]: https://github.com/nteract/mathjax

[^2_32]: https://github.com/pressbooks/pb-mathjax/blob/dev/LICENSE

[^2_33]: https://github.com/mkabumattar/reactify-katex/

[^2_34]: https://github.com/SamyPesse/react-mathjax

[^2_35]: https://github.com/Speech-Rule-Engine/speech-rule-engine/blob/develop/index.html

[^2_36]: https://github.com/Hermann-web/pix2tex

[^2_37]: https://github.com/NormXU/nougat-latex-ocr

[^2_38]: https://github.com/SimonYde/pix2tex.nix

[^2_39]: https://github.com/struglers/latexify

[^2_40]: https://github.com/Hermann-web/pix2tex/blob/main/README.md

[^2_41]: https://github.com/mohammadabidhafiz1294/Mathpix-OCR-Alternative

[^2_42]: https://github.com/breezedeus/Pix2Text-Mac

[^2_43]: https://github.com/breezedeus/Pix2Text-Mac/blob/main/LICENSE

[^2_44]: https://github.com/adereth/unicode-math

[^2_45]: https://github.com/tpapp/unicode-math-input

[^2_46]: https://mathparser.org/mxparser-tutorial/unicode-math/

[^2_47]: https://marketplace.visualstudio.com/items?itemName=czhang03.unicode-math-input

[^2_48]: https://www.youtube.com/watch?v=AgHWsfwmfNQ

[^2_49]: https://mathjax.github.io/MathJax-a11y/

