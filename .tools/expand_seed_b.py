#!/usr/bin/env python3
"""Second pass: top up to 500+ symbols."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SEED = ROOT / "src-tauri" / "data" / "seed.json"


def S(name_ko, short_ko, sym_type, *, unicode=None, latex=None,
      name_en=None, reading_ko=None, difficulty=2, flags=None,
      aliases=None, domains=None, examples=None):
    e = {
        "canonical_name_ko": name_ko, "short_meaning_ko": short_ko,
        "symbol_type": sym_type, "difficulty_level": difficulty,
        "flags": flags or [], "aliases": [list(a) for a in (aliases or [])],
        "domains": [list(d) for d in (domains or [])],
        "examples": [list(x) for x in (examples or [])],
    }
    if unicode is not None: e["unicode_char"] = unicode
    if latex is not None: e["latex_command"] = latex
    if name_en is not None: e["canonical_name_en"] = name_en
    if reading_ko is not None: e["reading_ko"] = reading_ko
    return e


ADDITIONS = [
    # math: orderings & extras
    S("선행", "정의 순서관계 ≺", "relation", unicode="≺", latex="\\prec",
      flags=["relation"], domains=[("math-algebra", "순서관계 ≺")]),
    S("후행", "정의 순서관계 ≻", "relation", unicode="≻", latex="\\succ",
      flags=["relation"], domains=[("math-algebra", "순서관계 ≻")]),
    S("선행이하", "≼", "relation", unicode="≼", latex="\\preceq",
      flags=["relation"], domains=[("math-algebra", "≼")]),
    S("후행이상", "≽", "relation", unicode="≽", latex="\\succeq",
      flags=["relation"], domains=[("math-algebra", "≽")]),
    S("배타적 OR", "XOR ⊻", "logic-symbol", unicode="⊻", latex="\\veebar",
      flags=["operator"], domains=[("logic", "XOR")]),
    S("NAND", "부정 곱 ⊼", "logic-symbol", unicode="⊼", latex="\\barwedge",
      flags=["operator"], domains=[("logic", "NAND")]),
    S("스타", "임의 별 연산", "operator", unicode="⋆", latex="\\star",
      flags=["operator"], domains=[("math-algebra", "별 연산")]),
    S("애스터리스크", "곱셈 변형", "operator", unicode="∗", latex="\\ast",
      flags=["operator"], domains=[("math-algebra", "별표 곱셈")]),
    S("바운드 박스 합", "직합 ⊞", "operator", unicode="⊞", latex="\\boxplus",
      flags=["operator"], domains=[("math-linalg", "박스 합")]),
    S("바운드 박스 곱", "직곱 ⊠", "operator", unicode="⊠", latex="\\boxtimes",
      flags=["operator"], domains=[("math-linalg", "박스 곱")]),

    # complex / number theory
    S("나누어떨어짐", "a | b", "relation", unicode="∣", latex="\\mid",
      flags=["relation"], domains=[("math-number", "약수 관계")]),
    S("나누지 않음", "a ∤ b", "relation", unicode="∤", latex="\\nmid",
      flags=["relation"], domains=[("math-number", "약수 아님")]),
    S("법연산 식", "a ≡ b (mod n)", "relation", latex="a \\equiv b \\pmod{n}",
      flags=["relation"], domains=[("math-number", "합동식")]),
    S("프라임 표기", "소수 집합 ℙ", "set-symbol", unicode="ℙ", latex="\\mathbb{P}",
      flags=["constant"], domains=[("math-number", "소수 집합")]),

    # calculus extras
    S("자코비안 |J|", "변수변환 자코비안 절댓값", "operator",
      latex="|J|", domains=[("math-calculus", "자코비안 절댓값")]),
    S("미분 형식 dx∧dy", "외적 미분 형식", "operator", latex="dx \\wedge dy",
      difficulty=3, domains=[("math-calculus", "미분 형식 wedge")]),
    S("외미분", "d (de-Rham)", "operator", latex="d",
      difficulty=3, domains=[("math-calculus", "외미분 연산자")]),
    S("크리스토펠 기호", "Γ^k_{ij}", "operator", latex="\\Gamma^k_{ij}",
      difficulty=3, domains=[("physics-general", "리만 기하 연결")]),
    S("리만 곡률", "R^a_{bcd}", "operator", latex="R^a_{bcd}",
      difficulty=3, domains=[("physics-general", "곡률 텐서")]),

    # physics extras
    S("벡터 r", "위치 벡터", "operator", latex="\\mathbf{r}",
      domains=[("physics-general", "위치 벡터")]),
    S("벡터 v", "속도 벡터", "operator", latex="\\mathbf{v}",
      domains=[("physics-general", "속도 벡터")]),
    S("벡터 a", "가속도 벡터", "operator", latex="\\mathbf{a}",
      domains=[("physics-general", "가속도 벡터")]),
    S("벡터 p", "운동량 벡터", "operator", latex="\\mathbf{p}",
      domains=[("physics-general", "운동량 벡터")]),
    S("푸인팅 벡터", "S = E × H", "operator", latex="\\mathbf{S}",
      difficulty=3, domains=[("physics-em", "전자기 에너지 흐름")]),
    S("자기 모멘트", "μ_m", "operator", latex="\\mu_m",
      domains=[("physics-em", "자기 모멘트")]),
    S("쌍극자 모멘트", "p = qd", "operator", latex="\\mathbf{p}_d",
      domains=[("physics-em", "전기 쌍극자")]),
    S("진동수 행 ν", "광자 진동수", "operator", latex="\\nu",
      domains=[("physics-quantum", "광자 진동수")]),
    S("주기 T_p", "회전 주기", "operator", latex="T_p",
      domains=[("physics-general", "주기")]),
    S("플랑크 시간", "t_P = √(ℏG/c⁵)", "constant", latex="t_P",
      flags=["constant"], difficulty=3, domains=[("physics-general", "플랑크 시간")]),
    S("플랑크 길이", "ℓ_P", "constant", latex="\\ell_P",
      flags=["constant"], difficulty=3, domains=[("physics-general", "플랑크 길이")]),
    S("플랑크 질량", "m_P", "constant", latex="m_P",
      flags=["constant"], difficulty=3, domains=[("physics-general", "플랑크 질량")]),

    # chemistry compounds extras
    S("황산", "H₂SO₄", "chemistry-notation", latex="H_2SO_4",
      domains=[("chemistry", "황산")]),
    S("염산", "HCl", "chemistry-notation", latex="HCl",
      domains=[("chemistry", "염산")]),
    S("질산", "HNO₃", "chemistry-notation", latex="HNO_3",
      domains=[("chemistry", "질산")]),
    S("수산화나트륨", "NaOH", "chemistry-notation", latex="NaOH",
      domains=[("chemistry", "강염기")]),
    S("산소 분자", "O₂", "chemistry-notation", latex="O_2",
      domains=[("chemistry", "산소 분자")]),
    S("질소 분자", "N₂", "chemistry-notation", latex="N_2",
      domains=[("chemistry", "질소 분자")]),
    S("수소 분자", "H₂", "chemistry-notation", latex="H_2",
      domains=[("chemistry", "수소 분자")]),
    S("오존", "O₃", "chemistry-notation", latex="O_3",
      domains=[("chemistry", "오존")]),
    S("포도당", "C₆H₁₂O₆", "chemistry-notation", latex="C_6H_{12}O_6",
      domains=[("biology", "포도당"), ("chemistry", "단당")]),
    S("에탄올", "C₂H₅OH", "chemistry-notation", latex="C_2H_5OH",
      domains=[("chemistry", "에탄올")]),
    S("벤젠", "C₆H₆", "chemistry-notation", latex="C_6H_6",
      domains=[("chemistry", "벤젠")]),
    S("메탄올", "CH₃OH", "chemistry-notation", latex="CH_3OH",
      domains=[("chemistry", "메탄올")]),

    # biology extras
    S("유사분열", "mitosis 표기", "biology-notation", latex="\\mathrm{M}",
      domains=[("biology", "M 단계")]),
    S("감수분열", "meiosis", "biology-notation", latex="\\mathrm{meiosis}",
      domains=[("biology", "감수분열")]),
    S("DNA 가닥 5'", "5'-말단", "biology-notation", latex="5'",
      domains=[("biology", "DNA 5' 말단")]),
    S("DNA 가닥 3'", "3'-말단", "biology-notation", latex="3'",
      domains=[("biology", "DNA 3' 말단")]),
    S("센티모르간", "유전 지도 cM", "biology-notation", latex="\\mathrm{cM}",
      domains=[("biology", "유전 거리")]),
    S("표준 자유에너지 변화", "ΔG°", "operator", latex="\\Delta G^{\\circ}",
      domains=[("chemistry", "표준 자유에너지")]),

    # earth / climate
    S("CO₂ 농도", "대기 CO₂ (ppm)", "earth-science",
      latex="\\mathrm{CO_2}", reading_ko="씨오 투",
      domains=[("earth-science", "대기 CO₂")]),
    S("PPM", "100만분의 1 농도", "unit-symbol", latex="\\mathrm{ppm}",
      domains=[("chemistry", "100만분의 1"), ("earth-science", "대기 농도")]),
    S("PPB", "10억분의 1 농도", "unit-symbol", latex="\\mathrm{ppb}",
      domains=[("chemistry", "10억분의 1")]),
    S("UTC", "협정 세계시", "earth-science", latex="\\mathrm{UTC}",
      domains=[("earth-science", "UTC")]),
    S("MSL", "평균 해수면", "earth-science", latex="\\mathrm{MSL}",
      domains=[("earth-science", "평균 해수면")]),

    # stats extras
    S("델타 함수 표본", "δ 임펄스", "operator", latex="\\delta",
      domains=[("statistics", "디랙 델타")]),
    S("Z-점수 표준화", "z = (x-μ)/σ", "operator", latex="z",
      domains=[("statistics", "표준화")]),
    S("표본 평균 분포", "X̄ ~ N(μ, σ²/n)", "operator",
      latex="\\bar{X}_n",
      domains=[("statistics", "중심극한정리")]),
    S("회귀잔차", "ε_i", "operator", latex="\\varepsilon_i",
      domains=[("statistics", "회귀 잔차")]),

    # logic extras
    S("진리값 0", "거짓 0", "logic-symbol", latex="0",
      flags=["constant"], domains=[("logic", "거짓")]),
    S("진리값 1", "참 1", "logic-symbol", latex="1",
      flags=["constant"], domains=[("logic", "참")]),
]


def main():
    data = json.loads(SEED.read_text(encoding="utf-8"))
    by_key = {
        (s.get("unicode_char"), s.get("latex_command")): s
        for s in data["symbols"]
    }
    added = 0
    skipped = 0
    for entry in ADDITIONS:
        key = (entry.get("unicode_char"), entry.get("latex_command"))
        if key in by_key:
            skipped += 1
            continue
        data["symbols"].append(entry)
        by_key[key] = entry
        added += 1
    SEED.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"added: {added}, skipped: {skipped}, total: {len(data['symbols'])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
