# 레이어에서 아틀라스 만들기

## 합성 방식

입력은 검수된 부위 PNG, 포즈별 소켓/가림 순서, 표정 레시피, 클립 시간/이벤트다. **이미지 생성기로 아틀라스 시트를 다시 만들지 않는다.** 부위 합성 → 완성 프레임 → 결정적 패킹으로 만들고 원본 레이어는 보존한다.

제작용 레이어와 런타임 렌더링은 다르다. 다음 중 실제 사용 방식을 명세한다.

- `baked`: 포즈·표정·고정 장비를 완성 프레임으로 합성한다. 스프라이트 수는 적지만 조합별 이미지 용량이 늘어난다.
- `hybrid`: 고정 바디/장비는 합성하고 독립 깜빡임·말하기/교체 장비만 별도 층으로 둔다. 현재 얼굴 오버레이와 유사하지만 새 규격과의 호환은 따로 확인한다.
- `layered`: 실시간 부위/의상 제어가 필요할 때 사용한다. 작은 눈·입까지 96×128 쿼드로 그리는 비용과 정렬을 측정한다. 원본이 레이어라는 이유만으로 선택하지 않는다.

눈×입×포즈×장비 전체 조합을 무조건 굽지 않는다. 실제 레시피만 합성하고 타이밍/이벤트가 달라도 같은 이미지 payload는 재사용한다.

## 원점과 trim

제작 원본은 공통 96×128 캔버스를 유지한다. 고정 셀은 단순하고 현재 카탈로그와 잘 맞지만 투명 여백이 남는다. trim은 바깥 투명 여백만 제거하고 `source_size`, `trim_offset`, 원래 좌표의 `pivot`을 보존한다. 렌더러가 이 복원을 지원할 때만 런타임 trim을 적용한다.

예: 원본 96×128, 보이는 bbox `[21,7,77,124)`면 rect 크기는 56×117, `trim_offset=[21,7]`, 원래 `pivot=[48,124]`, trim 후 피벗은 `[27,117]`이다. 피벗은 rect 끝이나 바깥에 있을 수 있다. UV rect는 페이지 좌표, 피벗/소켓은 원본 로컬 좌표다.

rect는 정수 `[x,y,width,height]`, bbox는 반열린 구간이다. trim을 원래 캔버스에 복원한 픽셀이 원본과 같아야 한다. 부품마다 중심을 재설정하지 않는다. 완전히 빈 선택층은 `hidden` 상태로 표현하고 0×0 텍스처로 만들지 않는다.

## 페이지와 경계

함께 노출되고 같은 sampler/material을 쓰는 항목을 우선 묶는다. 모든 장면을 거대 페이지 하나로 합치지 않는다. 초기 페이지 예산은 최대 **2048×2048**을 검토하되 기기·상주 메모리에 맞춰 결정한다. **4096은 보편적인 모바일 실패 경계가 아니다.** 실제 texture dimension limit과 메모리 예산을 확인한다.

기본은 nearest min/mag, 정수 표시 배율, mipmap 없음, 회전 패킹 없음이다. power-of-two는 소비자/형식/기기가 요구할 때 사용한다. UV 회전/trim을 소비자가 지원하지 않으면 패커도 사용하지 않는다.

기본 경계안은 rect 바깥 **extrude 1px**, 그 바깥 **투명 gutter 1px**다. 각 방향 총 2px를 예약하므로 인접 rect 사이 최소 간격은 4px다. rect는 내용만 가리키며 extrude/gutter를 포함하지 않는다. 페이지 가장자리도 같은 공간을 예약한다. 패커의 padding 용어가 다르면 이 정의로 환산한다. 이 값은 nearest/no-mipmap 시작안이며 mip/축소 필터를 쓰면 bleed를 다시 검증한다. clamp만으로 페이지 내부 이웃 색 번짐은 해결되지 않는다.

안정적인 상태 ID 정렬·패커 설정·원본 해시로 재빌드한다. 프레임 ID를 디렉터리 열거 순서에 맡기지 않는다. 보이는 RGBA가 같은 payload는 공유할 수 있지만 source_size/offset/pivot이 다르면 메타데이터는 분리한다. PNG 생성 시각 같은 비시각 정보 때문에 결과가 계속 바뀌지 않게 한다.

## 제작용 manifest

다음은 새 `atlas-manifest.json`의 최소 계약이다. 기존 게임 카탈로그가 아니며 adapter 없이 로드하지 않는다. 경로는 manifest 기준이다. `extrude_px`, `gutter_px`는 rect 바깥 각 방향의 공간이다.

```json
{
  "version": 1,
  "sampler": "nearest",
  "mipmaps": false,
  "extrude_px": 1,
  "gutter_px": 1,
  "pages": [{"id": "npc-0", "path": "atlas.png", "size": [512, 512]}],
  "sprites": [{
    "id": "neutral", "page": "npc-0", "rect": [2, 2, 56, 117],
    "source_size": [96, 128], "trim_offset": [21, 7], "pivot": [48, 124],
    "source": "composed/neutral.png", "rotated": false
  }],
  "clips": {"idle": {"loop": true, "frames": [{"sprite": "neutral", "duration_ms": 120}]}}
}
```

`source`는 패킹 전 완성 프레임이다. 클립의 참조/시간/이벤트와 이미지 rect를 분리한다. 같은 rect를 여러 sprite ID가 공유할 수 있지만 원점·피벗·이벤트는 잃지 않는다. 포즈별 소켓·가림 순서와 표정 채널 우선순위는 캐릭터 명세/연결된 pose 데이터에 보존한다.

## 기존 Bevy 소비자 경계

확인 시점 `crates/forge-npc/src/lib.rs`의 `Catalog`는 cell 크기 `Frame.rect`, foot/head, 초 단위 duration, clips를 사용한다. 추가 얼굴층은 mouths 4×1, eyes 3×1, hands 4×6 가정이 있다. 새 눈 5종·포즈 수·trim은 자동 호환되지 않는다. 통합할 때 현재 코드를 다시 확인한다.

`scripts/pack-npc.py`, `assets/manifest.json`도 기존 규격이다. 새 제작 명세를 이 스키마처럼 덮어쓰거나 검증을 우회하지 않는다. 통합이 요청되면 adapter/소비자 변경과 초↔ms 변환, 상태 ID 매핑을 별도 범위로 수행한다.

## 내보내기 검수

1. 부위 → 완성 프레임: 바탕 손실, 표정 외 변경, 두상·가림 순서를 확인한다.
2. 페이지 경계·rect 중첩·예약 공간, source_size/offset/pivot, 클립 참조·양수 시간을 검사한다. 회전은 기본 금지다.
3. rect를 꺼내 원래 캔버스로 복원한 보이는 RGBA가 원본과 같아야 한다. 투명 픽셀의 숨은 RGB는 무시한다.
4. 1배/4배와 실제 sampler에서 경계 bleed, 클립·소켓을 확인한다. 패킹만으로 드로우 콜 감소를 선언하지 않는다.
5. [sprite-optimization](../../sprite-optimization/SKILL.md)으로 정적 지표와 런타임 측정을 구분한다. PNG는 무손실로 보존하고 GPU 압축은 최종 전달 단계에서 지원/품질을 확인한다.

공식 자료: [Bevy nearest sampler](https://docs.rs/bevy/0.20.0/bevy/image/struct.ImageSamplerDescriptor.html), [wgpu texture limits](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html). 픽셀 보존과 성능 측정은 별개의 합격 조건이다.
