//! Opt-in diagnostic evidence, never a source of gameplay or animation state.
use crate::{Options, Session, spaces::WorkshopActor};
use bevy::{
    asset::UntypedAssetId,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        erased_render_asset::ErasedRenderAssets,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        texture::GpuImage,
        view::screenshot::Screenshot,
    },
    sprite_render::{MeshMaterial2d, PreparedMaterial2d, SpriteMeshMaterial},
};
use forge_npc::player::{Actor, Walker};
use std::{io::Write, path::PathBuf};

#[derive(Resource, Clone, ExtractResource, Default)]
#[extract_app(RenderApp)]
struct Frame {
    output: PathBuf,
    number: u32,
    stop_at: Option<u32>,
    continuous: bool,
    actors: Vec<(String, AssetId<Image>, Option<UntypedAssetId>)>,
}
#[derive(Default)]
struct Sequence {
    clip: String,
    remaining: u8,
}
pub fn install(app: &mut App) {
    let Some(path) = std::env::var_os("LITTLE_FORGE_TRACE_DIR") else {
        return;
    };
    let output = PathBuf::from(path);
    std::fs::create_dir_all(&output).expect("create forge trace directory");
    app.insert_resource(Frame {
        output,
        continuous: std::env::var_os("LITTLE_FORGE_TRACE_CONTINUOUS").is_some(),
        stop_at: std::env::var("LITTLE_FORGE_TRACE_STOP")
            .ok()
            .and_then(|s| s.parse().ok()),
        ..default()
    })
    .add_plugins(ExtractResourcePlugin::<Frame>::default())
    .add_systems(Last, sample);
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.add_systems(Render, gpu.in_set(RenderSystems::Cleanup));
    }
}
fn write(path: PathBuf, value: serde_json::Value) {
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open forge trace");
    writeln!(f, "{value}").expect("write forge trace");
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn sample(
    mut c: Commands,
    mut exit: MessageWriter<AppExit>,
    opts: Res<Options>,
    s: Res<Session>,
    server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mut frame: ResMut<Frame>,
    mut sequence: Local<Sequence>,
    actors: Query<
        (
            Entity,
            &Actor,
            &Walker,
            &Sprite,
            &Visibility,
            &Transform,
            Option<&MeshMaterial2d<SpriteMeshMaterial>>,
        ),
        With<WorkshopActor>,
    >,
    stations: Query<(
        Entity,
        &forge_npc::stations::StationLayer,
        &Sprite,
        &Visibility,
        &Transform,
    )>,
    cameras: Query<(Entity, &Camera, &Projection, &bevy::camera::RenderTarget)>,
    windows: Query<&Window>,
) {
    frame.number = opts.frames;
    if frame.stop_at.is_some_and(|end| opts.frames >= end) {
        exit.write(AppExit::Success);
    }
    frame.actors.clear();
    let actor_data: Vec<_> = actors.iter().map(|(e,a,w,sprite,v,t,material)| {
        frame.actors.push((format!("{e:?}"),sprite.image.id(),material.map(|m|m.0.id().untyped())));
        serde_json::json!({"entity":format!("{e:?}"),"clip":a.clip,"clip_frame":a.clip_frame,"elapsed":a.elapsed,"position":[w.pos.x,w.pos.y],"z":t.translation.z,"visibility":format!("{v:?}"),"image":format!("{:?}",sprite.image.id()),"cpu_loaded":images.contains(sprite.image.id()),"load":format!("{:?}",server.get_load_state(sprite.image.id())),"atlas":format!("{:?}",sprite.texture_atlas),"material":format!("{:?}",material.map(|m|m.0.id()))})
    }).collect();
    let station_data: Vec<_> = stations.iter().map(|(e,l,sprite,v,t)| serde_json::json!({"entity":format!("{e:?}"),"station":l.station,"layer":l.kind,"visibility":format!("{v:?}"),"z":t.translation.z,"cpu_loaded":images.contains(sprite.image.id())})).collect();
    let camera_data: Vec<_> = cameras.iter().map(|(e,c,p,target)|serde_json::json!({"entity":format!("{e:?}"),"active":c.is_active,"order":c.order,"viewport":format!("{:?}",c.viewport),"target":format!("{:?}",target),"projection":format!("{p:?}"),"clear":format!("{:?}",c.clear_color)})).collect();
    write(
        frame.output.join("main.jsonl"),
        serde_json::json!({"frame":frame.number,"tick":s.save.state.tick,"activity":format!("{:?}",s.save.state.activity()),"actors":actor_data,"stations":station_data,"cameras":camera_data,"windows":windows.iter().map(|w|[w.resolution.physical_width(),w.resolution.physical_height()]).collect::<Vec<_>>()}),
    );
    if let Ok((_, actor, _, _, _, _, _)) = actors.single() {
        if sequence.clip != actor.clip {
            sequence.clip = actor.clip.clone();
            sequence.remaining = 12;
        }
        if opts.frames > 90
            && (sequence.remaining > 0
                || opts.frames.is_multiple_of(120)
                || frame.continuous && (120..750).contains(&opts.frames))
        {
            sequence.remaining = sequence.remaining.saturating_sub(1);
            c.spawn(Screenshot::primary_window())
                .observe(crate::save_capture(
                    frame.output.join(format!("frame-{:04}.png", opts.frames)),
                ));
        }
    }
}
fn gpu(
    frame: Res<Frame>,
    images: Res<RenderAssets<GpuImage>>,
    materials: Res<ErasedRenderAssets<PreparedMaterial2d>>,
    cameras: Query<(
        &bevy::render::camera::ExtractedCamera,
        &bevy::render::view::ViewTarget,
    )>,
) {
    let actors: Vec<_> = frame.actors.iter().map(|(entity,image,material)|serde_json::json!({"entity":entity,"image_ready":images.get(*image).is_some(),"material":format!("{material:?}"),"material_ready":material.is_some_and(|id|materials.get(id).is_some())})).collect();
    let cameras: Vec<_> = cameras.iter().map(|(c,t)| serde_json::json!({"order":c.order,"index":c.sorted_camera_index_for_target,"viewport":format!("{:?}",c.viewport),"physical_target":format!("{:?}",c.physical_target_size),"texture_size":[t.main_texture().width(),t.main_texture().height()],"texture_id":format!("{:?}",t.main_texture_view().id()),"clear":format!("{:?}",c.clear_color),"output":format!("{:?}",c.output_mode)})).collect();
    write(
        frame.output.join("gpu.jsonl"),
        serde_json::json!({"frame":frame.number,"actors":actors,"cameras":cameras}),
    );
}
