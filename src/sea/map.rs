use bevy::prelude::*;
use bevy::ecs::bevy_utils::HashMap;
use std::{time::Duration};
use pirate_tilemap::{ChunkLayer, Chunk, TileMapBuilder, Layer, TileMapPlugin, LayerComponents, get_layer_components, AnimatedSyncMap, SCALING};
use super::player::PlayerPositionUpdate;
use super::worldgen::{CHUNK_SIZE, TILE_SIZE, generate_chunk};

//use super::player::{Player, FrictionType};
struct MapParam {
    seed : usize
} 

#[derive(Default)]
pub struct SeaHandles {
    base_islands_sheet : Handle<TextureAtlas>,
    sea_sheet : Handle<TextureAtlas>
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Sand(bool),
    Forest,
    Sea (bool), 
}
impl Default for TileKind {
    fn default() -> Self {
        TileKind::Sea(false)
    }
}

#[derive(Default, Clone, Copy)]
pub struct Tile {
    pub kind : TileKind,
    pub variant : u32,
    pub sprite_id : u32
}

pub struct SeaLayerMem {
    layer : LayerComponents,
}

pub struct SeaMapPlugin;
impl Plugin for SeaMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_plugin(TileMapPlugin)
        .add_startup_system(setup.system() )
        .add_startup_system(draw_chunks_system.system() )
        .init_resource::<Time>()
        .init_resource::<SeaHandles>()
        .add_resource(MapParam {seed : 12345})
        .add_resource(SeaLayerMem {layer : LayerComponents::default()})
        .add_system(draw_chunks_system.system())
        .add_system(despawn_chunk_system.system())
        ;
    }
}

fn get_sea_layer(handles : &ResMut<SeaHandles>) -> Layer {
    let mut tiles = Vec::new();
    let tile = pirate_tilemap::Tile::Animated(vec![1, 2, 3]);
    for x in 0..CHUNK_SIZE/4 {
        tiles.push(Vec::new());
        for _ in 0..CHUNK_SIZE/4 {
            tiles[x as usize].push(tile.clone())
        }
    }
    Layer {
        tiles,
        atlas_handle : handles.sea_sheet,
        anim_frame_time : Some(Duration::from_millis(500)),
        sync : true,
        num_frames : 3
    }
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    param : Res<MapParam>,
    mut sea : ResMut<SeaLayerMem>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut handles: ResMut<SeaHandles>,
    mut meshes : ResMut<Assets<Mesh>>, 
    mut materials : ResMut<Assets<ColorMaterial>>, 
) {
    //loading textures
    let texture_handle_map_spritesheet = asset_server.load("assets/sprites/sea/sheet.png").unwrap();
    let texture_handle_sea_spritesheet = asset_server.load("assets/sprites/sea/seaTileSheet.png").unwrap();
    
    //initializing the sea animation
    let island_atlas = TextureAtlas::from_grid(texture_handle_map_spritesheet, Vec2::new(64., 752.), 4, 47);
    let sea_atlas = TextureAtlas::from_grid(texture_handle_sea_spritesheet, Vec2::new(192., 64.), 3, 1);
    handles.base_islands_sheet = atlases.add(island_atlas);
    handles.sea_sheet = atlases.add(sea_atlas);
    let layer = get_sea_layer(&handles);
    sea.layer = get_layer_components(
        &*atlases,
        &mut *meshes, 
        &mut *materials, 
        &layer, 
        0, 
        &Transform::from_translation(Vec3::new(0., 0., 0.))
    ).0;
    let atlas_handle = handles.base_islands_sheet;
    //generating the first chunk
    let tiles = generate_chunk(0, 0, param.seed);
    //spawning entities
    //first chunk map
    commands.spawn(sea.layer.clone()).with(AnimatedSyncMap);
    let tilemap_builder = TileMapBuilder {
        layers : vec![
            Layer {
                tiles,
                atlas_handle,
                ..Default::default()
        }],
        layer_offset : 1,
        transform : Transform::from_translation(Vec3::new(0., 0., 0.)),
        chunk_x : 0, 
        chunk_y : 0,
    };
    commands
    .spawn((tilemap_builder,))
    ;
}

fn draw_chunks_system(
    mut commands: Commands,
    param : Res<MapParam>,
    pos_update : Res<PlayerPositionUpdate>,
    handles : Res<SeaHandles>,
    sea : Res<SeaLayerMem>,
    mut chunks : ResMut<HashMap<(i32, i32), Chunk>>,
) {
    if pos_update.changed_chunk {
        let chunk_x = pos_update.chunk_x;
        let chunk_y = pos_update.chunk_y;
        let surroundings = [(chunk_x + 1, chunk_y),
                                                 (chunk_x + 1, chunk_y + 1),
                                                 (chunk_x + 1, chunk_y - 1),
                                                 (chunk_x, chunk_y + 1),
                                                 (chunk_x, chunk_y - 1),
                                                 (chunk_x - 1, chunk_y + 1),
                                                 (chunk_x - 1, chunk_y),
                                                 (chunk_x - 1, chunk_y - 1),
        ];
        for (x, y) in surroundings.iter() {
            let mut sea_chunk = sea.layer.clone();
            sea_chunk.transform.translate(Vec3::new((TILE_SIZE*SCALING*CHUNK_SIZE*x) as f32,
            (TILE_SIZE*SCALING*CHUNK_SIZE*y) as f32, 
            0.));
            commands.spawn(sea_chunk).with(AnimatedSyncMap);
            if chunks.contains_key(&(*x, *y)) {
                let chunk = chunks.get_mut(&(*x, *y)).unwrap();
                    if chunk.drawn {
                        continue;
                    }
                    chunk.drawn = true;
                    for component in &mut chunk.bundles {
                        commands.spawn(
                            component.clone()
                        );
                    }
            }
            else {
                let tiles = generate_chunk(*x, *y, param.seed);
                let atlas_handle = handles.base_islands_sheet;
                let layers = vec!(
                    Layer {
                        tiles,
                        atlas_handle,
                        ..Default::default()
                    } 
                );
                let tilemap_builder = TileMapBuilder {
                    layers,
                    layer_offset : 1,
                    transform : Transform::from_translation(Vec3::new((TILE_SIZE*CHUNK_SIZE*x) as f32,
                    (TILE_SIZE*CHUNK_SIZE*y) as f32, 
                    0.)),
                    chunk_x : *x, 
                    chunk_y : *y,
                };
                commands
                .spawn((tilemap_builder,));
            }
            }
        }
    }

fn despawn_chunk_system(
    mut commands: Commands,
    pos_update : Res<PlayerPositionUpdate>,
    mut chunks : ResMut<HashMap<(i32, i32), Chunk>>,
    mut chunk_query : Query<(Entity, &Transform, &ChunkLayer)>,
) {
    if pos_update.changed_chunk {
        for (entity, tile_pos, _) in &mut chunk_query.iter() {
            let tile_pos = tile_pos.translation();
            let limit = (CHUNK_SIZE*TILE_SIZE*SCALING) as f32 * 2.5;
            if (tile_pos.x() - pos_update.get_x()).abs() > limit || (tile_pos.y() - pos_update.get_y()).abs() > limit {
                let chunk_x = (tile_pos.x()/(TILE_SIZE*SCALING*CHUNK_SIZE) as f32).floor() as i32;
                let chunk_y = (tile_pos.y()/(TILE_SIZE*SCALING*CHUNK_SIZE) as f32).floor() as i32;
                if let Some(chunk) = chunks.get_mut(&(chunk_x, chunk_y)) {
                    chunk.drawn = false;
                    commands.despawn(entity);
                }
                else {
                    println!("should not happen");
                }
                
            }
        }
    }
}

