# Smith's Codex — 장인의 도감과 작품의 흔적

[첫 유산](../mvp/FIRST_LEGACY.md) · [성장](PROGRESSION.md) · [무기](WEAPONS.md) · [세계 진행](WORLD_PROGRESSION.md)

## 목표

무기를 단순 제작·판매·소모하는 경험을 **기억에 남는 제작 기록과 장인으로서의 성취**로 바꾼다. 판매·납품·탐험 제공·Salvaging 이후에도 이미 달성한 기록은 사라지지 않는다. 무기뿐 아니라 농기구·주방 도구·목공 도구·가죽세공 도구도 포함한다.

## 아이템 유형별 기록

- 안정적 `item_definition_id`: 발견/해금/최초 제작 시각, 제작 횟수, 최고 품질과 기록 소유품 식별자, 제작 기술, 고객 피드백, 대표 작품.
- 개별 `crafted_item_id`: 실제 소유권·재료·기법·강화·제작자 서명. 거래 가능한 고유 제작물 추적과 분리.
- 제작 기록: 최초 제작 / 숙련 목표 / **현장 사용 평가** 의 단계. '증명'은 필요할 때만 열며 모든 도구에 무거운 업적 의무를 넣지 않는다.
- 외부 플레이어에게 판매된 무기도 원 제작자의 도감 기록에는 계속 남는다(실제 마켓 서버 소유권과 분리).

## 초기 UX

- 작은 목록에서는 아이템 실루엣/카테고리/진행 배지.
- 선택 상세: 제작한 횟수, 최고 품질, 연결 기술, 관련 연구 노드, 실제 NPC 사용 후 피드백, 첫 활약 기록.
- 품목의 선행조건이나 연구 단서를 누르면 Research Tree로 이동.
- 새 기록은 아늑하고 짧은 UI 피드백으로 전달. 매 제작마다 강제 팝업/포커스 없음.

## 초기 완성 기준

Chef Kitchen Knife → Dagger의 제작 이력부터 검증하고, Farmer Rake → Spear, Hunter가 제공받은 Bow/Arrows의 현장 평가까지 연결. 품질 등급 실제 계산기가 아직 없으면 임의의 가짜 최고 품질 데이터를 만들지 말고 필드를 미확정/미관측으로 표시한다.

## QA / 경계

보상 이벤트 중복·판매 후 기록 삭제·동일 아이템 재귀 등록·저장 복귀 누락 방지. Codex는 제작 가능 규칙의 출처가 아니다. 제작 자격은 연구와 기술 레벨이 담당하고 도감은 **이미 일어난 사실**을 기록한다.


## Expedition combat slice (2026-10-18)

The first expedition now uses deterministic five-wave automatic combat. See [combat rules](EXPEDITION.md) and the implementation boundary in [Hunter return](FIRST_EXPEDITION.md). Weapon IDs retain actual field-use and defeat counts; five-wave completion reopens the forest and awards eight timber once. Full research/technique progression and the broader Codex UI remain separate open work, not completion claims from this combat slice.
