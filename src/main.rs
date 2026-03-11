use bevy::prelude::*;

const TILE_SIZE: f32 = 32.0;
const MAP_WIDTH: i32 = 40;
const MAP_HEIGHT: i32 = 40;

fn main() {

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Podo Farm".into(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_camera, setup_tilemap, setup_player).chain())
        .add_systems(Update, (move_player, camera_follow).chain())
        .run();
}

// 컴포넌트
// 이 엔티티가 플레이어임
#[derive(Component)]
struct Player;

// 카메라
#[derive(Component)]
struct MainCamera;

// 타일 종류
#[derive(Component)]
#[derive(Clone, Copy, PartialEq)]
enum TileType {
    Grass,
    Dirt,
    Water,
}

impl TileType {
    fn color(&self) -> Color {
        match self {
            TileType::Grass => Color::srgb(0.2, 0.6, 0.2),
            TileType::Dirt => Color::srgb(0.5, 0.3, 0.1),
            TileType::Water => Color::srgb(0.2, 0.4, 0.8),
        }
    }

    fn is_walkable(&self) -> bool {
        match self {
            TileType::Grass => true,
            TileType::Dirt => true,
            TileType::Water => false,
        }
    }
}

#[derive(Resource)]
struct TileMap {
    tiles: Vec<Vec<TileType>>,
}

impl TileMap {
    // 월드 좌표를 타일 인덱스로 변환
    fn world_to_tile(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let tile_x = ((x / TILE_SIZE) + (MAP_WIDTH as f32 / 2.0)).floor() as i32;
        let tile_y = ((MAP_HEIGHT as f32 / 2.0) - (y / TILE_SIZE)).floor() as i32;

        // 경계 값을 맵 범위 내로 클램프
        let tile_x = tile_x.clamp(0, MAP_WIDTH - 1);
        let tile_y = tile_y.clamp(0, MAP_HEIGHT - 1);

        Some((tile_x as usize, tile_y as usize))
    }

    fn is_walkable(&self, x: f32, y: f32) -> bool {
        match self.world_to_tile(x, y) {
            Some((tx, ty)) => self.tiles[ty][tx].is_walkable(),
            None => false,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        MainCamera,
    ));
}

fn setup_tilemap(mut commands: Commands) {
    // 맵 데이터 생성
    let mut tiles = vec![vec![TileType::Grass; MAP_WIDTH as usize]; MAP_HEIGHT as usize];

    // 농장 땅
    for y in 5..15 {
        for x in 5..20 {
            tiles[y][x] = TileType::Dirt;
        }
    }

    for y in 10..20 {
        for x in 22..35 {
            tiles[y][x] = TileType::Dirt;
        }
    }

    // 연못
    for y in 20..26 {
        for x in 5..12 {
            tiles[y][x] = TileType::Water;
        }
    }

    // 강
    for y in 0..MAP_HEIGHT as usize {
        for x in 36..40 {
            tiles[y][x] = TileType::Water;
        }
    }

    // 타일 스폰
    for (y, row) in tiles.iter().enumerate() {
        for (x, tile_type) in row.iter().enumerate() {
            let position = Vec3::new(
                (x as f32 - MAP_WIDTH as f32 / 2.0 + 0.5) * TILE_SIZE,
                (MAP_HEIGHT as f32 / 2.0 - y as f32 - 0.5) * TILE_SIZE,
                0.0,
            );

            commands.spawn((
                Sprite {
                    color: tile_type.color(),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_translation(position),
            ));
        }
    }

    commands.insert_resource(TileMap {tiles});
}

fn setup_player(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.7, 0.5),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player,
    ));
}

// 매 프레임 실행
fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    tilemap: Res<TileMap>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let speed = 200.0;
    let dt = time.delta_secs();

    let player_half_size = 16.0;
    let map_half_width = (MAP_WIDTH as f32 / 2.0) * TILE_SIZE;
    let map_half_height = (MAP_HEIGHT as f32 / 2.0) * TILE_SIZE;

    let bound_x = map_half_width - player_half_size;
    let bound_y = map_half_height - player_half_size;

    for mut transform in &mut query {
        let cur_x = transform.translation.x;
        let cur_y = transform.translation.y;
        let mut new_x = cur_x;
        let mut new_y = cur_y;

        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            new_y += speed * dt;
        }

        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            new_y -= speed * dt;
        }

        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            new_x -= speed * dt;
        }

        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            new_x += speed * dt;
        }

        // 맵 경계 제한
        new_x = new_x.clamp(-bound_x, bound_x);
        new_y = new_y.clamp(-bound_y, bound_y);

        // 축 분리 충돌 체크: X와 Y를 독립적으로 검사하여 벽을 따라 슬라이딩 가능
        let hs = player_half_size;

        // X축 이동 체크
        let can_move_x = tilemap.is_walkable(new_x - hs, cur_y - hs)
            && tilemap.is_walkable(new_x + hs, cur_y - hs)
            && tilemap.is_walkable(new_x - hs, cur_y + hs)
            && tilemap.is_walkable(new_x + hs, cur_y + hs);

        if can_move_x {
            transform.translation.x = new_x;
        } else if new_x > cur_x {
            let snap = ((cur_x + hs) / TILE_SIZE).ceil() * TILE_SIZE - hs - 0.01;
            if tilemap.is_walkable(snap + hs, cur_y - hs)
                && tilemap.is_walkable(snap + hs, cur_y + hs) {
                transform.translation.x = snap;
            }
        } else if new_x < cur_x {
            let snap = ((cur_x - hs) / TILE_SIZE).floor() * TILE_SIZE + hs;
            if tilemap.is_walkable(snap - hs, cur_y - hs)
                && tilemap.is_walkable(snap - hs, cur_y + hs) {
                transform.translation.x = snap;
            }
        }

        let final_x = transform.translation.x;

        // Y축 이동 체크
        let can_move_y = tilemap.is_walkable(final_x - hs, new_y - hs)
            && tilemap.is_walkable(final_x + hs, new_y - hs)
            && tilemap.is_walkable(final_x - hs, new_y + hs)
            && tilemap.is_walkable(final_x + hs, new_y + hs);

        if can_move_y {
            transform.translation.y = new_y;
        } else if new_y > cur_y {
            let snap = ((cur_y + hs) / TILE_SIZE).ceil() * TILE_SIZE - hs;
            if tilemap.is_walkable(final_x - hs, snap + hs)
                && tilemap.is_walkable(final_x + hs, snap + hs) {
                transform.translation.y = snap;
            }
        } else if new_y < cur_y {
            let snap = ((cur_y - hs) / TILE_SIZE).floor() * TILE_SIZE + hs + 0.01;
            if tilemap.is_walkable(final_x - hs, snap - hs)
                && tilemap.is_walkable(final_x + hs, snap - hs) {
                transform.translation.y = snap;
            }
        }
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for mut camera_transform in &mut camera_query {
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
    }
}