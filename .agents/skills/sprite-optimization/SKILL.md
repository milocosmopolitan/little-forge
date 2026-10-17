---
name: sprite-optimization
description: 스프라이트와 아틀라스의 배칭·드로우 콜, 투명 여백·레이어 오버드로우, 텍스처 크기·GPU 메모리·압축을 검수한다. 이 저장소의 Bevy 렌더러와 실제 타깃 조건을 확인하고 픽셀 품질·피벗을 보존한다. 정적 이미지 지표와 측정된 런타임 성능을 구분한다.
---

# Sprite Optimization

에셋/아틀라스 검수나 최적화에 사용한다. 사람 캐릭터 제작에는 [human-character-gen](../human-character-gen/SKILL.md), 픽셀 보존에는 [pixel-art-gen](../pixel-art-gen/SKILL.md)을 함께 적용한다. [엔진별 주의점과 공식 근거](references/runtime-notes.md)를 필요한 부분만 읽는다.

## 먼저 확인할 조건

현재 엔진/버전, 렌더 경로·sampler·blend/material, 타깃 기기, 동시에 보이는 캐릭터 수, 실제 표시 크기, 아틀라스/레이어 전략을 확인한다. 현재 저장소는 Cargo 기반 Bevy이며 Unity의 import 설정이 아니다. 문서의 과거 버전보다 실제 Cargo와 소비 코드가 우선한다.

사용자가 에셋 검수만 요청했다면 자동 읽기 검사를 진행하고 런타임 재설계·카탈로그 교체·GPU 압축 변환으로 범위를 늘리지 않는다. 타깃이나 성능 캡처가 없으면 정적 검수를 완료하고 런타임 검증을 미측정으로 남긴다. 추정값을 실측값으로 부르지 않는다.

## 1. 배칭과 드로우 콜

- 함께 노출되며 같은 material/sampler/blend/렌더 패스를 쓰는 스프라이트를 같은 페이지로 묶는지 본다. 다른 장면의 거대 아틀라스로 상주 메모리를 늘리지 않는다.
- 같은 아틀라스는 배칭의 기회다. z 정렬, 다른 텍스처 페이지·셰이더·클리핑·UI 경계·블렌드 상태가 배치를 나눌 수 있다. 이미지 수/페이지 수를 드로우 콜 수로 환산하지 않는다.
- 제작 층은 보존하고 런타임에는 고정 부위를 bake하거나 필요한 눈·입만 남기는 hybrid를 비교한다. draw call 감소와 픽셀 처리량 감소는 별개이며 같은 배치의 여러 레이어도 겹쳐 그릴 수 있다.
- 변경 전후 같은 장면·카메라·해상도·캐릭터 수·타임라인에서 렌더 캡처/프로파일러로 실제 draw calls, batches, CPU/GPU frame time을 비교한다. 워밍업·캡처 도구·기기와 샘플 구간을 기록한다. 렌더 캡처가 없으면 감소량은 미측정이다.

## 2. 투명 여백과 오버드로우

- 네이티브 이미지와 개별 atlas rect의 alpha bbox, 불투명 비율, trim 후 사각형 면적을 측정한다. 페이지 전체의 빈 공간은 메모리/패킹 효율이고 실제 그려지는 sprite rect의 투명 여백과는 다르다.
- trim은 `source_size`, `trim_offset`, pivot/소켓을 보존하며 지원되는 소비자에만 적용한다. 제작용 눈·입 원본은 공통 캔버스를 유지한다. 원본/복원 픽셀 비교를 통과하지 못하면 채택하지 않는다.
- 현재 Bevy 경로에서 Unity `Mesh Type=Tight`를 설정했다고 보고하지 않는다. 필요하면 trim 쿼드나 커스텀 메쉬의 실제 지원/비용을 검토한다. Tight mesh도 정점·삼각형 증가 비용이 있어 무조건 빠르지 않다.
- 몸 전체 크기의 작은 눈·입 쿼드, 불필요한 가림층, 숨은 스프라이트 제출, UI 중첩을 본다. 실제 화면의 레이어 겹침/픽셀 처리량은 GPU 캡처나 해당 엔진의 진단 도구로 확인한다. 정적 alpha 비율을 화면 오버드로우 횟수로 부르지 않는다.

## 3. 텍스처 크기·메모리·압축

- 초기 페이지 예산은 2048×2048을 검토한다. 디바이스 texture dimension limit과 실제 상주 페이지·메모리 예산으로 결정하며 4096을 보편적인 모바일 실패 기준으로 쓰지 않는다.
- PNG 파일 크기와 GPU 메모리는 다르다. 비압축 RGBA8 base level은 `width×height×4` bytes다. 2048²는 16MiB, 4096²는 64MiB다. 전체 mip chain은 큰 정사각형에서 약 4/3배이며 정확 계산 시 모든 레벨을 합한다. CPU 사본·staging·alignment·다중 버퍼는 별도다.
- PPU는 월드 크기 대응값이며 그 자체로 텍스처 메모리를 줄이지 않는다. 이 프로젝트에서는 카메라·transform·native pixel의 표시 비율을 확인한다. 픽셀 아트는 nearest·정수 정렬로 시작하고 mipmap 여부를 별도로 확인한다. nearest 설정이 mipmap 생성을 끈다는 뜻은 아니다.
- PNG는 무손실이므로 무손실 PNG 재저장을 손실성 이중 압축으로 부르지 않는다. 단, 팔레트 축소/리샘플링은 손실일 수 있다. JPEG나 이미 손실 GPU 압축된 데이터를 다시 디코드·압축하는 경로는 피하고 무손실 마스터에서 최종 전달용 압축을 1회 파생한다.
- ASTC/ETC2/BC3(DXT5)는 후보이지 전 기기 공통 설정이 아니다. 하드웨어·그래픽 API·엔진 빌드의 format 지원/확장·fallback을 확인한다. 블록 압축의 눈·수염·1px 윤곽·alpha와 atlas 경계 오염을 실제 표시 배율에서 비교한다. 작은 픽셀 아트는 RGBA8이 더 적절할 수 있다. 압축을 강제하지 않는다.

## 읽기 전용 정적 감사

Pillow가 필요하다. RTK 환경에서는 아래처럼 실행한다. 스크립트는 이미지를 수정하지 않으며 JSON을 stdout에 출력한다.

```sh
rtk proxy python3 .agents/skills/sprite-optimization/scripts/audit_sprites.py assets/npc/farmer/base-neutral.png assets/npc/farmer/neutral.png
rtk proxy python3 .agents/skills/sprite-optimization/scripts/audit_sprites.py --manifest path/to/atlas-manifest.json
```

manifest는 [제작용 계약](../human-character-gen/references/create-atlas.md)을 사용한다. 기존 `character.json`이나 임의 패커 JSON을 이 스키마로 간주하지 않는다. 먼저 adapter가 필요한지 확인한다. `--page-budget 2048`은 검토 경고 기준이고 `--device-max-size`는 실제 조회한 하드웨어 상한이 있을 때만 지정한다. `--gpu-format`의 메모리는 지원 여부를 확인하지 않은 형식별 계산값이다.

스크립트는 파일/크기/alpha/색 수/내용 중복, rect 경계·겹침·예약 공간, trim 원점 복원과 source 픽셀 비교, 클립 참조/시간을 검사한다. extrude/gutter의 **픽셀 내용**, shader sampling, 실제 배칭·오버드로우·피벗 렌더링은 검증하지 않는다. 새 패커는 별도 경계 픽셀 검사와 런타임 검수를 수행한다. 최적화 결과는 [create-atlas.md](../human-character-gen/references/create-atlas.md)의 원본 보존 조건도 통과해야 한다.

## 결과 보고

발견 사항마다 근거(파일/rect/코드/캡처), 영향, 수정 방법, 검증 상태를 남긴다. 보고 상태는 `구조 통과`, `검토 필요`, `런타임 미측정`, `실측 확인`으로 구분한다. 정적 경고를 성능 실패로 단정하지 않는다.

최소 지표는 페이지 수·크기/상주 추정 bytes, rect/alpha 점유율, 중복, 기본/trim 면적, sampler/mipmap/압축 지원 확인 여부다. 실제 최적화 변경을 했다면 픽셀/원점/클립 무손실 비교와 같은 장면의 전후 성능을 함께 보고한다. 미측정인 성능 항목을 구조 통과로 대체하지 않는다.
