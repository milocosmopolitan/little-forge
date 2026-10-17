# 엔진과 근거

## 현재 저장소에 적용할 때

2026-10-16 확인 기준 `Cargo.toml`은 Bevy `=0.20.0`이다. desktop과 npc-preview 진입점은 `ImagePlugin::default_nearest()`를 설정한다. 이것만으로 개별 이미지 sampler·mip level·실제 draw call은 확인되지 않는다. 작업 때 현재 코드를 다시 확인한다.

`crates/forge-npc/src/lib.rs`는 기본 스프라이트와 추가 얼굴/손 레이어를 사용한다. `Catalog::validate`는 프레임 크기와 cell의 일치를 요구하고 얼굴층 레이아웃에 개수 가정이 있다. 제작용 레이어/trim atlas를 자동 호환으로 간주하지 않는다. 대조 경로는 `apps/desktop/src/main.rs`, `apps/npc-preview/src/main.rs`, `crates/forge-npc/src/lib.rs`, `scripts/pack-npc.py`다.

현재 Cargo의 이미지 형식 기능에는 PNG가 명시되어 있다. GPU 압축 전달용 파일이 필요하면 포맷 로더·빌드 기능·변환 경로·fallback을 실제로 확인한다. 형식별 메모리 계산기를 실행한 것을 해당 형식의 게임 로딩 성공으로 해석하지 않는다.

Unity 전용 설정인 Sprite Importer의 PPU/Mesh Type, Sprite Atlas Tight Packing, Overdraw 뷰를 Bevy에서 수행한 것처럼 기록하지 않는다. Bevy에서도 같은 문제를 검사하되 실제 camera/transform/sampler, 쿼드/메쉬, GPU 캡처와 배치 경계를 찾는다.

## 범용 주장 보정

- Atlas는 텍스처 전환을 줄일 수 있지만 material·정렬 등의 배치 경계를 없애지는 않는다. 페이지를 하나로 만들었다고 draw call=1이 되지 않는다.
- 투명 여백 비율은 낭비 후보 지표다. 실제 GPU 비용은 화면 크기·셰이더·가림·제출 메쉬·레이어 수에 따라 달라진다.
- PPU는 픽셀과 월드 단위의 대응이다. 원본 해상도/GPU format/mip count가 같으면 PPU만 바꿔 메모리를 줄였다고 할 수 없다.
- 4096은 기기 공통 성능 경계가 아니다. 지원 dimension limit은 크기의 허용 여부이고, 대역폭/상주 메모리 비용의 적정성은 별도다.
- PNG의 무손실 재압축과 손실 블록 압축을 구별한다. 압축 포맷 선택은 지원 여부·정확한 block 크기·작은 디테일 손상까지 확인한다.

## 공식 자료

- [Bevy 0.20 nearest sampler](https://docs.rs/bevy/0.20.0/bevy/image/struct.ImageSamplerDescriptor.html): min/mag 필터와 mip 설정의 별도 필드를 확인한다.
- [wgpu Limits](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html): `max_texture_dimension_2d` 등 실제 디바이스 제한을 확인한다.
- [Unity Sprite Importer](https://docs.unity3d.com/Manual/texture-type-sprite.html): PPU의 월드 단위 의미와 Full Rect/Tight 차이. Tight의 세밀한 메쉬는 정점 증가 비용이 있다.
- [Unity Sprite Atlas 설정](https://docs.unity3d.com/Manual/sprite/atlas/sprite-atlas-reference.html): packing·padding·filtering·플랫폼 texture 설정을 구분한다.
- [Unity texture compression](https://docs.unity3d.com/Manual/texture-compression-formats.html): 형식별 플랫폼 지원과 용량/품질을 확인한다.
- [W3C PNG](https://www.w3.org/TR/png/): PNG는 무손실 형식이다. 양자화 등 PNG 저장 전 처리의 손실까지 보장하지는 않는다.

사용자가 제공한 YouTube 링크 두 개는 이번 조회에서 본문을 가져오지 못했다. 영상의 발언을 검증된 근거로 재인용하지 않았으며 위 공식 자료와 저장소를 기준으로 규칙을 작성했다.
