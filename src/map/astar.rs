use super::Map;
use pathfinding::prelude::astar;
use rltk::{BaseMap, DistanceAlg, Point};

pub fn a_star_search(start: Point, end: Point, map: &Map) -> Option<(Vec<Point>, i32)> {
    astar(
        &start,
        |p| {
            let p_idx = map.xy_idx(p.x, p.y);
            map.get_available_exits(p_idx)
                .iter()
                .map(|(idx, cost)| {
                    (
                        Point::new(
                            *idx as i32 % map.width as i32,
                            *idx as i32 / map.width as i32,
                        ),
                        (*cost * 256.) as i32,
                    )
                })
                .collect::<Vec<(Point, i32)>>()
        },
        |p| (DistanceAlg::PythagorasSquared.distance2d(start, *p) * 256.) as i32,
        |p| *p == end,
    )
}
