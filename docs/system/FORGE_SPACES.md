# Forge Spaces — 세 공간·작업·진열 용량 계약 (Epic #86)

[홈](../../README.md) · [아키텍처](../../ARCHITECTURE.md) · [제작](CRAFTING.md) · [마을 거래](VILLAGE_COMMERCE.md) · [QA 증거](../qa/FORGE_SPACES.md)

> 상태: **구현 계약**. 2026-10-18 main `340940a` 조사와 사용자 결정(외형만 다른 캐릭터 선택, 같은 상태를
> 공유하는 세 화면, 일반 완성품과 진열 용량의 통합, 작업 시작 시 결과 슬롯 예약)을 코드로 옮긴 기준이다.
> 수치는 **조정값**이며 `assets/showroom.json`에서 바꾼다.

## 1. 하나의 대장간, 세 가지 화면

| 화면 | 창 모드 | 역할 | 코드 |
|---|---|---|---|
| 외부 (Exterior) | 소형 960×180 기본, 확장 시에도 선택 가능 | 다른 일을 하며 띄워두는 디오라마. 건물 플레이트(작업 중 화로 점등/굴뚝 연기), 대기 중인 대장장이, 방문객, 한 줄 상태 칩, 문 위 `Enter` 영역 | `apps/desktop/src/spaces.rs` `SceneOf(Exterior)` |
| 작업장 (Workshop) | 확장 960×540 | 기물 5종과 선택한 대장장이. 기물을 누르면 작업 메뉴(큐 등록·광석 구매), `Manage`로 기존 재료/식량 장부 | `SceneOf(Workshop)`, `drive_workshop` |
| 진열장 (Showroom) | 확장 960×540 | 완성품 슬롯 창(진열대 탭·공통 슬롯·배지)과 상세 패널(판매/보관 전환, 목공소 이동) | `SceneOf(Showroom)`, `draw_showroom` |

- **화면 선택(`Space`)과 창 모드(`Shell.expanded`)와 마을 내비게이션(`Navigation.screen`)은 서로 다른 상태다.** 소형 창은 항상 외부를 보여주고, 확장 창은 `Space`를 보여주며, 마을 지도/상점은 그 위에 덮인다. 대장간 외부는 마을 지도의 `Outside`(탐험) 목적지와 다르다.
- 입력: `Tab` 소형→작업장, `1/2/3` 외부/작업장/진열장(홈 화면에서만), `Esc` 작업장 서랍 닫기 → 축소, `M` 마을 지도. 창 투명도·항상 위·클릭 통과 설정은 바꾸지 않는다.
- 세 화면은 `Session.save.state` 하나만 읽는다. 뷰 생성/폐기는 타이머나 보상 처리기를 등록하지 않으며, 보이지 않는 동안에도 `simulate`는 동일하게 진행한다.

## 2. 캐릭터 선택

- `State.smith: Option<Smith>` (`female` = Eira, `male` = Bram). 외형만 다르며 능력·경제 차이는 없다.
- 새 게임: `타이틀 → New Game → Start forging → Choose(두 초상, 대기/깜빡임) → Pick`. **Pick 순간에만** `Command::ChooseSmith`와 첫 저장이 발생한다. Back은 아무것도 쓰지 않는다.
- 계속하기: 선택이 저장돼 있으면 묻지 않는다. 스키마 3 이하 세이브는 `smith = None`으로 올라오고 Continue에서 **한 번** Choose를 거친다. 진행(골드·재고·큐·RNG)은 그대로다.
- `ChooseSmith`는 `smith`가 이미 있으면 `InvalidState`로 거부한다. 예전 `assets/forge/visitor.png`는 방문객 역할로만 남고 플레이어로 쓰지 않으며, `smith.png`는 더 이상 그리지 않는다.

## 3. 작업 지시와 모션

- 한 명의 대장장이가 한 번에 한 작업을 한다. 큐(`life.queue`, 최대 32)는 등록 시 광석을 차감(예약)하고, **시작 시 결과 슬롯을 예약**한다.
- 레시피 단계는 권위 있는 남은 작업량에서 **파생**된다: `State::work_stage()` → `Good::stages()` (철제 단검: 가열 30% → 망치질 50% → 담금질 20%). 뷰 전환·오프라인 복귀는 단계를 다시 재생하지 않는다.
- `State::activity()`가 표시 계약이다: `Working{station, progress}` / `Waiting(reason)` / `Idle` / `Sleeping` / `Eating` / `Drinking`.
- 어댑터(`forge_npc::player`)는 `animation.json`(기물·방향·loop_range·이벤트)과 `atlas-manifest.json`(트림·피벗)을 소비한다. 이동은 고정 접근점 사이의 직선(`stations::spot`)이며 범용 길찾기는 없다. `hammer_contact`·`water_enter`·`heat_exit`·`parts_place`는 `Actor.events`에 쌓이는 소리/효과 표식일 뿐 결과를 지급하지 않는다.
- Alpha 01 연구대는 실제 `DaggerStudy` 작업과 Research 창에 연결된다. 분해는 아직 사용할 수 없다고 표시한다. 전체 연구 그래프와 분해 규칙은 이 범위에 포함하지 않는다.

## 4. 유한 완성품 용량 — 진열장

`State.showroom: Showroom { racks, slots, overflow, config }`

| 슬롯 | 의미 | 용량 점유 | 방문객 구매 |
|---|---|---|---|
| `Empty` | 새 작업이 예약할 수 있음 | 아니오 | — |
| `Reserved` | 진행 중 작업의 결과 자리 | 예 | 아니오 |
| `Displayed(good)` | 판매 진열 | 예 | 예 |
| `Held(good)` | 판매 보류(연구·분해·보관용) | 예 | 아니오 |

- 제품 1개 = 슬롯 1개. 숨은 무제한 보관함은 없다. `daggers == showroom.goods()`가 불변식이다.
- **예약 원장**: `Craft`는 슬롯 예약 → 광석 차감 → 작업 시작을 한 명령 안에서 원자적으로 수행하며 빈자리가 없으면 `Error::DisplayFull`로 거부한다. 큐 작업은 `life_step`에서 예약에 성공할 때만 시작하고, 실패하면 광석을 다시 차감하지 않고 기다린다(`CraftBlocked` 이벤트 1회, `Activity::Waiting`).
- 완료는 `Reserved → Displayed`를 정확히 한 번 수행한다. 판매는 `Displayed → Empty` 한 번, 골드 +가격 한 번. `SetForSale`은 `Displayed ↔ Held`만 바꾸며 `Reserved`/`Empty`는 거부한다.
- 용량 변경(구매·업그레이드)은 슬롯을 **추가만** 하고 기존 물건·예약을 옮기지 않는다. 대기 중인 큐는 다음 초에 다시 평가된다.
- 수탁 의뢰품·탐험 대여품은 이 슬롯에 들어오지 않는다. #77/#81 소유권 계약이 확정되면 별도 보관 규칙을 붙이고, 일반 자동 판매/분해 경로를 재사용하지 않는다.
- 유한 진열은 **동시 보관량** 제한이다. 자동 제작·판매가 계속되는 한 누적 수입은 계속 늘며, 오프라인 상한 8시간 규칙은 바꾸지 않았다.

## 5. 목공소 진열대 (#95)

`Command::BuyRack`, `Command::UpgradeRack { rack }` — 골드+목재 차감과 슬롯 증가를 한 명령에서 처리하고, 버튼은 렌더 시점의 명령 ID를 들고 있어 중복 입력이 두 번 차감하지 않는다.

| 항목 | 초안값 (`assets/showroom.json`) |
|---|---|
| 티어별 슬롯 | 4 / 6 / 8 |
| 설치 상한 | 3대 |
| 진열대 구매 | 30 골드 + 목재 4 |
| 업그레이드 (1→2 / 2→3) | 20 골드 + 목재 2 / 40 골드 + 목재 4 |

판매 속도·가격 버프·자유 배치는 없다. 목공소 `Furniture` 탭에서 구매/업그레이드하며 Timber 상품과 별개 ID다.

## 6. 저장과 이전 (현재 스키마 5)

- Alpha 01의 스키마 5는 별도 progression 상태를 추가한다. 스키마 1–4의 기존 단검 제작 권한은 보존하며 과거 XP/도감 기록은 만들지 않는다. [Alpha 검증](../qa/PLAYABLE_ALPHA_01.md) 참고.

- 스키마 4 = 3 + `state.smith` + `state.showroom`. 스키마 4 파일에 `showroom`이 없으면 후보에서 제외한다(기존 commerce 규칙과 동일). 더 높은 스키마는 거부한다.
- 스키마 3 → 4: `daggers`를 기본 진열대(4칸)에 `Displayed`로 채우고, 진행 중 작업이 있으면 `Reserved` 한 칸을 확보한다(밀려난 기존 단검은 `overflow`로). 나머지는 `overflow`에 보존한다.
- `overflow`는 이전 전용 상태다: 플레이어 소유이며 **먼저 팔리고**, 소진 전까지 새 일반 제작을 막는다(`BlockReason::LegacyOverflow`). 삭제·임의 자동판매·숨은 보관함은 없다. 런타임에 늘지 않는다.
- 큐에서 이미 차감된 광석은 다시 차감하지 않는다(시작 시 슬롯만 예약).

## 7. 책임 분담

| 영역 | 소유 |
|---|---|
| 슬롯/예약/판매/용량/선택 | `forge-core` (`showroom.rs`, `craft.rs`, `spaces.rs`, `lib.rs`) |
| 초안 밸런스 | `forge-content::showroom_config()` ← `assets/showroom.json` |
| 플레이어 재생·기물 배치 | `forge-npc::player`, `forge-npc::stations` (Bevy, forge-core 미의존) |
| 세 화면·입력·UI | `apps/desktop/src/spaces.rs` + 기존 `village.rs`/`lifestyle.rs`/`title.rs` |
| 공용 위젯 | `forge-ui` (`item_slot`, `SkinFrame`, `SkinSlot`, 배지 색) |
| 저장 슬롯/설정/종료 | 기존 #35/#37/#39/#40/#41 경로 재사용, 별도 저장 시스템 없음 |
| 연구/분해/의뢰/대여 | #72/#73 — 이 에픽은 경계만 남긴다 |

## 8. 제외

WebP 전환, 자유 배치/범용 길찾기, 테트리스 인벤토리, 캐릭터 능력 차이, 멀티플레이, 진열 가구별 무기 그림(무기는 아이콘 하나를 제작·진열·도감에서 재사용한다).
