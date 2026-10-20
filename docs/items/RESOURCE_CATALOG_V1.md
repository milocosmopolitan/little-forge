# 기초 자원 카탈로그 v1

상태: **PR #126에서 에셋 40종 병합 완료; 획득·가공 레시피는 설계안**. 정본: [material-catalog.json](../../assets/items/materials/material-catalog.json), [아트 규칙](../design/MATERIAL_ATLAS_V1.md).

모든 아이콘은 32×32 RGBA, 1px 투명 여백, 최대 16색. 아틀라스는 256×192 (8×6).

| 종류 | ID | 개수 |
|---|---|---:|
| 광석 | iron_ore, copper_ore, tin_ore, lead_ore, silver_ore, gold_ore, zinc_ore | 7 |
| 주괴 | iron_ingot, copper_ingot, tin_ingot, lead_ingot, silver_ingot, gold_ingot, zinc_ingot, steel_ingot, bronze_ingot, brass_ingot | 10 |
| 원목 | pine_log, oak_log, ash_log, walnut_log, cherry_log, mahogany_log, yew_log, birch_log | 8 |
| 나뭇가지 | wood_branch | 1 |
| 생가죽 | deer_hide, boar_hide, wolf_pelt, bear_hide, tiger_pelt | 5 |
| 가공가죽 | deer_leather, boar_leather, wolf_leather, bear_leather, tiger_leather | 5 |
| 연료·부산물 | charcoal, coal, wood_ash, pine_resin | 4 |

## 가공·획득 제안
- 광석 + 연료 → 대응 주괴. 철 주괴 + 숯/석탄 + 제강 → 강철. 구리+주석 → 청동; 구리+아연 → 황동. 합금 광석은 없음.
- 원목/나뭇가지의 제한 산소 탄화 → 숯, 완전 연소 → 재(wood_ash). 소나무에서 송진 채취. 석탄은 광산 원정 루팅.
- 생가죽/모피 → 같은 종의 가공가죽. 늑대와 멧돼지는 현재 탐험 몬스터와 연결 가능; 사슴·곰·호랑이의 실제 획득처는 미구현.
- iron/steel: 절삭·구조 부품; bronze/brass: 리벳·결합구·장식·일부 주조 머리; lead: 무게추; silver/gold: 장식·후속 마법 매개. **모든 금속으로 칼날을 만들지 않는다.**
- 원목은 수종 특성을 유지. 일반 나뭇가지는 종별로 나누지 않는다. 특수 완드 가지는 향후 별도 원정 아이템.
- `ash_log`는 물푸레나무, `wood_ash`는 재. `ash` 단독 ID 금지.

기존 코어의 통합 `Timber`/`Leather` 재고와 새 ID의 마이그레이션은 별도 결정 사항이다. 아이콘 보유가 실제 게임 획득 가능성을 뜻하지 않는다. 수량·가격·제련 시간은 밸런스 미확정.

관련: [부품](INITIAL_PARTS_V1.md) · [레시피](INITIAL_RECIPES_V1.md) · [완제품](INITIAL_FINISHED_GOODS_V1.md).
