use super::{BuilderChain, BuilderMap, InitialMapBuilder, Position, TileType};
use rltk::RandomNumberGenerator;
use std::collections::HashSet;

pub fn town_builder(
    depth: i32,
    width: i32,
    height: i32,
    _rng: &mut RandomNumberGenerator,
) -> BuilderChain {
    let mut chain = BuilderChain::new(depth, width, height);
    chain.start_with(TownBuilder::new());
    chain
}

pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data);
    }
}

#[derive(Debug)]
enum BuildingTag {
    Pub,
    Temple,
    Blacksmith,
    Clothier,
    Alchemist,
    PlayerHouse,
    Hovel,
    Abandoned,
    Unassigned,
}

impl TownBuilder {
    pub fn new() -> Box<Self> {
        Box::new(TownBuilder {})
    }

    pub fn build_rooms(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        self.grass_layer(build_data);
        self.water_and_piers(rng, build_data);
        let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);
        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.add_paths(build_data, &doors);

        let exit_idx = build_data.map.xy_idx(build_data.map.width - 5, wall_gap_y);
        build_data.map.tiles[exit_idx] = TileType::DownStairs;
        build_data.take_snapshot();

        let building_size = self.sort_buildings(&buildings);
        self.building_factory(rng, build_data, &buildings, &building_size);

        // Make visible for screenshot
        for t in build_data.map.visible_tiles.iter_mut() {
            *t = true;
        }
        build_data.take_snapshot();
    }

    fn grass_layer(&mut self, build_data: &mut BuilderMap) {
        // We'll start with a nice layer of grass
        for t in build_data.map.tiles.iter_mut() {
            *t = TileType::Grass;
        }
        build_data.take_snapshot();
    }

    fn water_and_piers(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut n = (rng.roll_dice(1, 65535) as f32) / 65535f32;
        let mut water_width = Vec::new();
        for y in 0..build_data.map.height {
            let n_water = (f32::sin(n) * 10.0) as i32 + 14 + rng.roll_dice(1, 6);
            water_width.push(n_water);
            n += 0.1;
            for x in 0..n_water {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::DeepWater;
            }
            for x in n_water..n_water + 3 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::ShallowWater;
            }
        }
        build_data.take_snapshot();

        // Add piers
        for _i in 0..rng.roll_dice(1, 4) + 6 {
            let y = rng.roll_dice(1, build_data.map.height) - 1;
            for x in 2 + rng.roll_dice(1, 6)..water_width[y as usize] + 4 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::WoodFloor;
            }
            build_data.take_snapshot();
        }
    }

    fn town_walls(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) -> (HashSet<usize>, i32) {
        let mut available_building_tiles = HashSet::new();
        let wall_gap_y = rng.roll_dice(1, build_data.map.height - 9) + 5;
        for y in 1..build_data.map.height - 2 {
            if !(y > wall_gap_y - 4 && y < wall_gap_y + 4) {
                let idx = build_data.map.xy_idx(30, y);
                build_data.map.tiles[idx] = TileType::Wall;
                build_data.map.tiles[idx - 1] = TileType::Floor;
                let idx_right = build_data.map.xy_idx(build_data.map.width - 2, y);
                build_data.map.tiles[idx_right] = TileType::Wall;
                for x in 31..build_data.map.width - 2 {
                    let gravel_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[gravel_idx] = TileType::Gravel;
                    if y > 2 && y < build_data.map.height - 1 {
                        available_building_tiles.insert(gravel_idx);
                    }
                }
            } else {
                for x in 30..build_data.map.width {
                    let road_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[road_idx] = TileType::Road;
                }
            }
        }
        build_data.take_snapshot();

        for x in 30..build_data.map.width - 1 {
            let idx_top = build_data.map.xy_idx(x, 1);
            build_data.map.tiles[idx_top] = TileType::Wall;
            let idx_bottom = build_data.map.xy_idx(x, build_data.map.height - 2);
            build_data.map.tiles[idx_bottom] = TileType::Wall;
        }
        build_data.take_snapshot();

        (available_building_tiles, wall_gap_y)
    }

    fn buildings(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        available_building_tiles: &mut HashSet<usize>,
    ) -> Vec<(i32, i32, i32, i32)> {
        let mut buildings = Vec::new();
        let mut n_buildings = 0;
        let mut n_attempts = 0;
        while n_buildings < 12 && n_attempts < 2000 {
            n_attempts += 1;
            let bx = rng.roll_dice(1, build_data.map.width - 32) + 30;
            let by = rng.roll_dice(1, build_data.map.height) - 2;
            let bw = rng.roll_dice(1, 8) + 4;
            let bh = rng.roll_dice(1, 8) + 4;
            let mut possible = true;
            for y in by..by + bh {
                for x in bx..bx + bw {
                    if x < 0
                        || x > build_data.map.width - 1
                        || y < 0
                        || y > build_data.map.height - 1
                    {
                        possible = false;
                    } else {
                        let idx = build_data.map.xy_idx(x, y);
                        if !available_building_tiles.contains(&idx) {
                            possible = false;
                        }
                    }
                }
            }
            if possible {
                n_buildings += 1;
                buildings.push((bx, by, bw, bh));
                for y in by..by + bh {
                    for x in bx..bx + bw {
                        let idx = build_data.map.xy_idx(x, y);
                        build_data.map.tiles[idx] = TileType::WoodFloor;
                        available_building_tiles.remove(&idx);
                        available_building_tiles.remove(&(idx + 1));
                        available_building_tiles.remove(&(idx + build_data.map.width as usize));
                        available_building_tiles.remove(&(idx - 1));
                        available_building_tiles.remove(&(idx - build_data.map.width as usize));
                    }
                }
                build_data.take_snapshot();
            }
        }

        // Outline buildings
        let map_clone = build_data.map.clone();
        for y in 2..map_clone.height - 2 {
            for x in 31..map_clone.width - 2 {
                let idx = map_clone.xy_idx(x, y);
                if map_clone.tiles[idx] == TileType::WoodFloor {
                    let mut non_floor_neighbors = 0;
                    if map_clone.tiles[idx - 1] != TileType::WoodFloor {
                        non_floor_neighbors += 1;
                    }
                    if map_clone.tiles[idx + 1] != TileType::WoodFloor {
                        non_floor_neighbors += 1;
                    }
                    if map_clone.tiles[idx - map_clone.width as usize] != TileType::WoodFloor {
                        non_floor_neighbors += 1;
                    }
                    if map_clone.tiles[idx + map_clone.width as usize] != TileType::WoodFloor {
                        non_floor_neighbors += 1;
                    }
                    if non_floor_neighbors > 0 {
                        build_data.map.tiles[idx] = TileType::Wall;
                    }
                }
            }
        }
        build_data.take_snapshot();

        buildings
    }

    fn add_doors(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &mut Vec<(i32, i32, i32, i32)>,
        wall_gap_y: i32,
    ) -> Vec<usize> {
        let mut doors = Vec::new();
        for (bx, by, bw, bh) in buildings.iter() {
            let door_x = bx + 1 + rng.roll_dice(1, bw - 3);
            let cy = by + (bh / 2);
            let idx = if cy > wall_gap_y {
                // Door on the north wall
                build_data.map.xy_idx(door_x, *by)
            } else {
                build_data.map.xy_idx(door_x, by + bh - 1)
            };
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Door".to_string()));
            doors.push(idx);
        }
        build_data.take_snapshot();

        doors
    }

    fn add_paths(&mut self, build_data: &mut BuilderMap, doors: &[usize]) {
        let mut roads = Vec::new();
        for y in 0..build_data.map.height {
            for x in 0..build_data.map.width {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::Road {
                    roads.push(idx);
                }
            }
        }

        build_data.map.populate_blocked();
        for door_idx in doors.iter() {
            let mut nearest_roads = Vec::new();
            let door_pt = rltk::Point::new(
                *door_idx as i32 % build_data.map.width as i32,
                *door_idx as i32 / build_data.map.width as i32,
            );
            for r in roads.iter() {
                nearest_roads.push((
                    *r,
                    rltk::DistanceAlg::PythagorasSquared.distance2d(
                        door_pt,
                        rltk::Point::new(
                            *r as i32 % build_data.map.width as i32,
                            *r as i32 / build_data.map.width as i32,
                        ),
                    ),
                ))
            }
            nearest_roads.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let destination = nearest_roads[0].0;
            let path = rltk::a_star_search(*door_idx, destination, &mut build_data.map);
            if path.success {
                for step in path.steps.iter() {
                    let idx = *step as usize;
                    build_data.map.tiles[idx] = TileType::Road;
                    roads.push(idx);
                }
            }
            build_data.take_snapshot();
        }
    }

    fn sort_buildings(
        &mut self,
        buildings: &[(i32, i32, i32, i32)],
    ) -> Vec<(usize, i32, BuildingTag)> {
        let mut building_size: Vec<(usize, i32, BuildingTag)> = Vec::new();
        for (i, (_bx, _by, bw, bh)) in buildings.iter().enumerate() {
            building_size.push((i, bw * bh, BuildingTag::Unassigned));
        }
        building_size.sort_by(|a, b| b.1.cmp(&a.1));

        for (i, b) in building_size.iter_mut().enumerate() {
            b.2 = match i {
                0 => BuildingTag::Pub,
                1 => BuildingTag::Temple,
                2 => BuildingTag::Blacksmith,
                3 => BuildingTag::Clothier,
                4 => BuildingTag::Alchemist,
                5 => BuildingTag::PlayerHouse,
                _ => BuildingTag::Hovel,
            }
        }
        let last_index = building_size.len() - 1;
        building_size[last_index].2 = BuildingTag::Abandoned;

        building_size
    }

    fn building_factory(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &[(i32, i32, i32, i32)],
        building_index: &[(usize, i32, BuildingTag)],
    ) {
        for (i, _size, build_type) in building_index.iter() {
            let building = &buildings[*i];
            match build_type {
                BuildingTag::Pub => self.build_pub(building, build_data, rng),
                BuildingTag::Temple => self.build_temple(building, build_data, rng),
                BuildingTag::Blacksmith => self.build_smith(building, build_data, rng),
                BuildingTag::Clothier => self.build_clothier(building, build_data, rng),
                BuildingTag::Alchemist => self.build_alchemist(building, build_data, rng),
                BuildingTag::PlayerHouse => self.build_my_house(building, build_data, rng),
                BuildingTag::Hovel => self.build_hovel(building, build_data, rng),
                _ => {}
            }
        }
    }

    fn random_building_spawn(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
        to_place: &mut Vec<&str>,
        player_idx: usize,
    ) {
        to_place.reverse(); // So it is easy to pop() in order
        let (bx, by, bw, bh) = *building;
        for y in by..by + bh {
            for x in bx..bx + bw {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor
                    && idx != player_idx
                    && rng.roll_dice(1, 3) == 1
                {
                    if let Some(entity_tag) = to_place.pop() {
                        build_data.spawn_list.push((idx, entity_tag.to_string()));
                    }
                };
            }
        }
    }

    fn build_pub(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place the Player
        let (pub_x, pub_y, pub_w, pub_h) = *building;
        let player_x = pub_x + pub_w / 2;
        let player_y = pub_y + pub_h / 2;
        let player_idx = build_data.map.xy_idx(player_x, player_y);
        build_data.starting_position = Some(Position {
            x: player_x,
            y: player_y,
        });

        // Place other items
        let mut to_place = vec![
            "Barkeep",
            "Shady Salesman",
            "Patron",
            "Patron",
            "Keg",
            "Table",
            "Chair",
            "Table",
            "Chair",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, player_idx);
    }

    fn build_temple(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place = vec![
            "Priest",
            "Parishioner",
            "Parishioner",
            "Chair",
            "Chair",
            "Candle",
            "Candle",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, usize::MAX);
    }

    fn build_smith(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place = vec![
            "Blacksmith",
            "Anvil",
            "Water Trough",
            "Weapon Rack",
            "Armor Stand",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, usize::MAX);
    }

    fn build_clothier(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place = vec!["Clothier", "Cabinet", "Table", "Loom", "Hide Rack"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, usize::MAX);
    }

    fn build_alchemist(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place = vec!["Alchemist", "Chemistry Set", "Dead Thing", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, usize::MAX);
    }

    fn build_my_house(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place = vec!["Mom", "Bed", "Cabinet", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, usize::MAX);
    }

    fn build_hovel(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place = vec!["Peasant", "Bed", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, usize::MAX);
    }
}
