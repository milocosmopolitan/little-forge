# World Progression — 대장장이의 작품이 세계를 바꾸는 방식

[첫 유산](../mvp/FIRST_LEGACY.md) · [도감](SMITHS_CODEX.md) · [탐험](FIRST_EXPEDITION.md) · [마을](VILLAGE_COMMERCE.md)

## 문제와 해결

아이템을 팔고 골드만 늘어나면 왜 더 좋은 제작품을 만들고 싶은지 이유가 약하다. **도감(과거), 기술/연구(성장), 세계 상태(다음 목표)**를 세 축으로 연결한다.

마을의 생활 도구 공급과 사냥꾼의 모험이 **새 공급품, 주문, 대사, 연구 기회, 지역 접근**을 만든다. 연출은 실제 시스템 기록으로부터 발생해야 하며 단순 무작위 칭찬을 금지한다.

## 첫 지역: 오래된 숲길 — 스토리 상태 예시

1. `threat_reported`: 주민이 길이 위험해 목재 공급이 막혔다고 말한다.
2. `support_prepared`: 적격 장비와 보급품을 준비해 Hunter 파견 가능.
3. `hunter_returned`: 탐험 결과에 따른 보고가 발생하고 무기의 기여 평가가 기록된다.
4. `path_reopened`: 필요한 조건을 충족하면 숲길의 위협이 완화되어 새 재료/후속 주문 접근 가능.

이벤트 단계·요구 횟수·보상 아이템은 콘텐츠 시안이며 자동으로 확정 스펙이 되지 않는다. **플레이어가 제작한 아이템 때문에 세계가 달라지는 진행 방식**은 확정된 방향이다.

## 설계 규칙

- 지역 상태는 명시적 상태 머신/마일스톤으로 관리; 탐험 성공 1회로 모든 지역 자동 해금되지 않는다.
- 플레이어가 이전에 만든 아이템의 기여 기록은 Codex와 세계 상태 모두에 연결될 수 있다.
- 진행되지 않은 상태를 보여주되 다음 목표가 무엇인지 알 수 있게 한다.
- 지역 상태에 따른 NPC 대사/상점 접근 변경은 **미리 정의된 콘텐츠**로 처리한다. 전세계 동적 경제 시뮬레이션은 없음.
- 상태 변화와 보상은 정확히 한 번; 저장/로드 후 복원; 오프라인 진행 및 실패는 결정적이어야 한다.
- 기존 마을 주민 관계와 **별개의 무조건 전역 평판 시스템**을 강제하지 않는다.

## UI 및 QA

마을 지도에 상태 뱃지·신규 주문 표시, 상세 화면에는 왜 변화했는지 근거와 다음 조건을 표시한다. 현재 UI overlay epic #58의 지도/상점 UI와 충돌하지 않는 통합 작업으로 처리. 변화 전후 골든/플레이 시나리오 캡처.


## Expedition combat slice (2026-10-18)

The first expedition now uses deterministic five-wave automatic combat. See [combat rules](EXPEDITION.md) and the implementation boundary in [Hunter return](FIRST_EXPEDITION.md). Weapon IDs retain actual field-use and defeat counts; five-wave completion reopens the forest and awards eight timber once. Full research/technique progression and the broader Codex UI remain separate open work, not completion claims from this combat slice.
