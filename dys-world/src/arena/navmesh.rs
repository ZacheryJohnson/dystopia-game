use std::fmt::Display;
use std::sync::{Arc, Mutex};

use ordered_float::{self, OrderedFloat};

use petgraph::algo;
use petgraph::visit::EdgeRef;
use petgraph::graphmap::UnGraphMap;
use rand::Rng;
use rapier3d::prelude::*;
use rapier3d::na::vector;

use super::feature::NavmeshPathingType;
use super::Arena;

#[derive(Clone)]
pub struct ArenaNavmeshConfig {
    /// How granular of a navmesh should we generate?
    /// If =1, there will be one navmesh node per unit, so a square arena of 20x20 units would have 400 navmesh nodes.
    /// If <1, there will be more than one navmesh node per unit.
    pub unit_resolution: f32,
}

impl Default for ArenaNavmeshConfig {
    fn default() -> Self {
        Self { 
            unit_resolution: 1.0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArenaNavmeshNode {
    x_pos: OrderedFloat<f32>,
    y_pos: OrderedFloat<f32>,
    z_pos: OrderedFloat<f32>,
}

impl ArenaNavmeshNode {
    fn from_point(point: Point<f32>) -> ArenaNavmeshNode {
        ArenaNavmeshNode {
            x_pos: OrderedFloat::from(point.x),
            y_pos: OrderedFloat::from(point.y),
            z_pos: OrderedFloat::from(point.z),
        }
    }

    fn as_point(&self) -> Point<f32> {
        point![*self.x_pos, *self.y_pos, *self.z_pos]
    }
}

impl Display for ArenaNavmeshNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("({}, {}, {})", self.x_pos, self.y_pos, self.z_pos).as_str())
    }
}

pub struct ArenaNavmesh {
    graph: UnGraphMap<ArenaNavmeshNode, f32>,
    config: ArenaNavmeshConfig
}

impl ArenaNavmesh {
    // ZJ-TODO: refactor
    //          it's costly to re-iterate over all nodes to add edges
    pub fn new_from(arena: Arc<Mutex<Arena>>, config: ArenaNavmeshConfig) -> ArenaNavmesh {
        let arena = arena.lock().unwrap();
        let pathable_arena_features = arena
            .all_features()
            .iter()
            .filter(|filter| filter.pathing_type() == NavmeshPathingType::Generate);

        let mut graph = UnGraphMap::<ArenaNavmeshNode, f32>::new();

        // Add all pathable features
        for feature in pathable_arena_features {
            let origin = feature.origin();
            let Some(shape) = feature.shape() else {
                continue;
            };

            let shape_isometry = Isometry::new(origin.to_owned(), vector![0., 0., 0.]);
            let aabb = shape.compute_aabb(&shape_isometry);
            let vertices = aabb.vertices();

            let min_x = vertices.iter().min_by(|first, second| first.x.partial_cmp(&second.x).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get min_x").x;
            let max_x = vertices.iter().max_by(|first, second| first.x.partial_cmp(&second.x).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get max_x").x;
            let min_y = vertices.iter().min_by(|first, second| first.y.partial_cmp(&second.y).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get min_y").y;
            let max_y = vertices.iter().max_by(|first, second| first.y.partial_cmp(&second.y).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get max_y").y;
            let min_z = vertices.iter().min_by(|first, second| first.z.partial_cmp(&second.z).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get min_z").z;
            let max_z = vertices.iter().max_by(|first, second| first.z.partial_cmp(&second.z).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get max_z").z;

            let mut curr_z = min_z;
            while curr_z <= max_z {
                let mut curr_x = min_x;
                while curr_x <= max_x {
                    let mut curr_y = max_y;
                    while curr_y >= min_y {
                        let curr_point = point![curr_x, curr_y, curr_z];
                        if shape.contains_point(&shape_isometry, &curr_point) {
                            let node = ArenaNavmeshNode::from_point(curr_point);
                            graph.add_node(node);

                            break;
                        }

                        curr_y -= config.unit_resolution;
                    }
                    curr_x += config.unit_resolution;
                }                 
                curr_z += config.unit_resolution;
            }
        }

        // Remove nodes where unpathable features exist
        let unpathable_arena_features = arena
            .all_features()
            .iter()
            .filter(|filter| filter.pathing_type() == NavmeshPathingType::Block);
        for feature in unpathable_arena_features {
            let origin = feature.origin();
            let Some(shape) = feature.shape() else {
                continue;
            };

            let shape_isometry = Isometry::new(origin.to_owned(), vector![0., 0., 0.]);
            let aabb = shape.compute_aabb(&shape_isometry);
            let vertices = aabb.vertices();

            let min_x = vertices.iter().min_by(|first, second| first.x.partial_cmp(&second.x).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get min_x").x;
            let max_x = vertices.iter().max_by(|first, second| first.x.partial_cmp(&second.x).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get max_x").x;
            let min_y = vertices.iter().min_by(|first, second| first.y.partial_cmp(&second.y).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get min_y").y;
            let max_y = vertices.iter().max_by(|first, second| first.y.partial_cmp(&second.y).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get max_y").y;
            let min_z = vertices.iter().min_by(|first, second| first.z.partial_cmp(&second.z).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get min_z").z;
            let max_z = vertices.iter().max_by(|first, second| first.z.partial_cmp(&second.z).unwrap_or(std::cmp::Ordering::Equal)).expect("failed to get max_z").z;

            let mut curr_z = min_z;
            while curr_z <= max_z {
                let mut curr_x = min_x;
                while curr_x <= max_x {
                    let mut curr_y = min_y;
                    while curr_y <= max_y {
                        let curr_point = point![curr_x, curr_y, curr_z];
                        if shape.contains_point(&shape_isometry, &curr_point) {
                            if let Some(node) = ArenaNavmesh::get_closest_node(&graph, curr_point, config.unit_resolution) {
                                graph.remove_node(node);
                            }
                        }

                        curr_y += config.unit_resolution;
                    }
                    curr_x += config.unit_resolution;
                }                 
                curr_z += config.unit_resolution;
            }
        }
        

        // Add edges between nodes
        let mut new_edges = vec![];
        for node in graph.nodes() {
            const CARDINAL_NEIGHBOR_WEIGHT: f32 = 1.0;
            const DIAGONAL_NEIGHBOR_WEIGHT: f32 = 1.7;
            let neighbor_points: Vec<(Point<f32>, f32)> = vec![
                (node.as_point() + vector![-config.unit_resolution, 0.0, 0.0], CARDINAL_NEIGHBOR_WEIGHT),
                (node.as_point() + vector![config.unit_resolution, 0.0, 0.0], CARDINAL_NEIGHBOR_WEIGHT),
                (node.as_point() + vector![0.0, 0.0, -config.unit_resolution], CARDINAL_NEIGHBOR_WEIGHT),
                (node.as_point() + vector![0.0, 0.0, config.unit_resolution], CARDINAL_NEIGHBOR_WEIGHT),
                (node.as_point() + vector![-config.unit_resolution, 0.0, -config.unit_resolution], DIAGONAL_NEIGHBOR_WEIGHT),
                (node.as_point() + vector![-config.unit_resolution, 0.0, config.unit_resolution], DIAGONAL_NEIGHBOR_WEIGHT),
                (node.as_point() + vector![config.unit_resolution, 0.0, -config.unit_resolution], DIAGONAL_NEIGHBOR_WEIGHT),
                (node.as_point() + vector![config.unit_resolution, 0.0, config.unit_resolution], DIAGONAL_NEIGHBOR_WEIGHT),
            ];

            for (neighbor_point, weight) in neighbor_points {
                let maybe_neighbor_node = ArenaNavmeshNode::from_point(neighbor_point);
                if graph.contains_node(maybe_neighbor_node) {
                    new_edges.push((node, maybe_neighbor_node, weight));
                };
            }
        }

        for (node, neighbor, weight) in new_edges {
            graph.add_edge(node, neighbor, weight);
        }

        ArenaNavmesh {
            graph,
            config
        }
    }

    /// Attempts to create a path from one point to another point. Returns an empty vector if a path cannot be made.
    pub fn create_path(&self, from: Point<f32>, to: Point<f32>) -> Vec<Point<f32>> {
        let start_node = ArenaNavmesh::get_closest_node(&self.graph, from, self.config.unit_resolution);
        let end_node = ArenaNavmesh::get_closest_node(&self.graph, to, self.config.unit_resolution);
        
        let Some(start) = start_node else {
            return vec![];
        };

        let Some(end) = end_node else {
            return vec![];
        };

        let node_path = self.get_path_between_nodes(start, end);

        node_path
            .iter()
            .map(|node| node.as_point())
            .collect()
    }

    pub fn get_next_point(&self, from: Point<f32>, to: Point<f32>) -> Option<Point<f32>> {
        // ZJ-TODO: refactor. We shouldn't assume ground = 0.0 - what if there's ramps?
        let grounded_from = point![from.x, 0.0, from.z];
        let grounded_to = point![to.x, 0.0, to.z];

        // ZJ-TODO: refactor - we shouldn't reconstruct a new path each time, we should do it once and cache it
        let path = self.create_path(grounded_from, grounded_to);
        if path.len() < 2 {
            return None;
        }

        Some(path.iter()
            .skip(1)
            .next()
            .unwrap()
            .to_owned())
    }

    fn get_path_between_nodes(
        &self,
        from: ArenaNavmeshNode,
        to: ArenaNavmeshNode
    ) -> Vec<ArenaNavmeshNode> {
        // To introduce suboptimal pathing for combatants,
        // we'll add a random vector to A*'s normal heuristic algorithm of euclidean distance.

        // Generate a random vector. This isn't a prod-ready solution, as this can produce illogical vectors
        // (such as the exact opposite direction of where we want to go).
        // We'd fix this by comparing the cross product of the random vector with the euclidean vector,
        // and if a very small negative number, flipping the random vector (eg multiplying by -1).
        let random_vector = vector![
            rand::thread_rng().gen_range(-1.0..1.0),
            0.0,
            rand::thread_rng().gen_range(-1.0..1.0),
        ].normalize();

        // Generate a random amount of bias. This isn't a prod-ready solution, and should instead
        // read the stats from the pathing combatant, where combatants with good "game sense" have a smaller factor
        // (and thus are less affected by the potentially suboptimal vector).
        let suboptimal_decision_bias_factor = rand::thread_rng().gen_range(0.0..1.0);
        let suboptimal_bias_vector = random_vector * suboptimal_decision_bias_factor;

        let astar_result = algo::astar(
            &self.graph, 
            from, 
            |node| node.as_point() == to.as_point(), 
            |edge| *edge.weight(), 
            |node| {
                let euclidean_vector = to.as_point() - node.as_point();
                let scaled_suboptimal_vector = suboptimal_bias_vector * euclidean_vector.magnitude();
                (euclidean_vector + scaled_suboptimal_vector).magnitude()
            }
        );

        let Some((_total_cost, path)) = astar_result else {
            return vec![];
        };

        path
    }

    /// This function is **expensive**. Should not be used when constructing the navmesh graph, and only for client requests (like [create_path](ArenaNavmesh::create_path)).
    fn get_closest_node(graph: &UnGraphMap<ArenaNavmeshNode, f32>, point: Point<f32>, unit_resolution: f32) -> Option<ArenaNavmeshNode> {
        graph
            .nodes()
            .filter(|node| (node.as_point() - point).magnitude() <= unit_resolution)
            .min_by(|first, second| (first.as_point() - point).magnitude().partial_cmp(&(second.as_point() - point).magnitude()).unwrap_or(std::cmp::Ordering::Equal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_defaults() -> (Arc<Mutex<Arena>>, ArenaNavmeshConfig) {
        let test_arena = Arena::new_with_testing_defaults();
        let test_config = ArenaNavmeshConfig {
            unit_resolution: 1.0,
        };

        (Arc::new(Mutex::new(test_arena)), test_config)
    }

    fn test_f32_eq(a: f32, b: f32) {
        assert!((a - b).abs() <= f32::EPSILON)
    }

    #[test]
    fn test_get_closest_node_valid_point() {        
        let (test_arena, test_config) = test_defaults();
        let unit_resolution = test_config.unit_resolution;

        let navmesh = ArenaNavmesh::new_from(test_arena, test_config);
        let closest_node = ArenaNavmesh::get_closest_node(&navmesh.graph, point![1.1, 0.0, 1.3], unit_resolution);

        assert!(closest_node.is_some());

        test_f32_eq(*closest_node.unwrap().x_pos, 1.0_f32);
        test_f32_eq(*closest_node.unwrap().y_pos, 0.0_f32);
        test_f32_eq(*closest_node.unwrap().z_pos, 1.0_f32);
    }

    #[test]
    fn test_get_closest_node_invalid_point() {        
        let (test_arena, test_config) = test_defaults();
        let unit_resolution = test_config.unit_resolution;

        let navmesh = ArenaNavmesh::new_from(test_arena, test_config);
        let closest_node = ArenaNavmesh::get_closest_node(&navmesh.graph, point![-1.0, 0.0, 1.3], unit_resolution);

        assert!(closest_node.is_none());
    }

    #[test]
    fn test_get_path_between_nodes() {
        let (test_arena, test_config) = test_defaults();
        let unit_resolution = test_config.unit_resolution;

        let navmesh = ArenaNavmesh::new_from(test_arena, test_config);

        let start_node = ArenaNavmesh::get_closest_node(&navmesh.graph, point![1.0, 0.0, 1.0], unit_resolution).expect("failed to get start node");
        let end_node = ArenaNavmesh::get_closest_node(&navmesh.graph, point![1.0, 0.0, 5.0], unit_resolution).expect("failed to get end node");

        let path = navmesh.get_path_between_nodes(start_node, end_node);

        assert!(!path.is_empty());
        assert!(*path.first().unwrap() == start_node);
        assert!(*path.last().unwrap() == end_node);
    }

    #[test]
    fn test_get_path_between_nodes_prefers_diagonals() {        
        let (test_arena, test_config) = test_defaults();
        let unit_resolution = test_config.unit_resolution;

        let navmesh = ArenaNavmesh::new_from(test_arena, test_config);

        let start_node = ArenaNavmesh::get_closest_node(&navmesh.graph, point![1.0, 0.0, 1.0], unit_resolution).expect("failed to get start node");
        let end_node = ArenaNavmesh::get_closest_node(&navmesh.graph, point![2.0, 0.0, 2.0], unit_resolution).expect("failed to get end node");

        let path = navmesh.get_path_between_nodes(start_node, end_node);

        // An ideal path from (1.0, 0.0, 1.0) -> (2.0, 0.0, 2.0) with unit resolution of 1.0 should be exactly one node away (diagonally by (1.0, 0.0, 1.0)), 
        // meaning the total path should be only the start node and only the end node.
        assert!(path.len() == 2);

        assert!(*path.first().unwrap() == start_node);
        assert!(*path.last().unwrap() == end_node);
    }    

    #[test]
    fn test_get_path_between_nodes_with_smaller_resolution() {        
        let (test_arena, mut test_config) = test_defaults();
        test_config.unit_resolution = 0.5;
        let unit_resolution = test_config.unit_resolution;

        let navmesh = ArenaNavmesh::new_from(test_arena, test_config);

        let start_node = ArenaNavmesh::get_closest_node(&navmesh.graph, point![1.0, 0.0, 1.0], unit_resolution).expect("failed to get start node");
        let end_node = ArenaNavmesh::get_closest_node(&navmesh.graph, point![1.0, 0.0, 5.0], unit_resolution).expect("failed to get end node");

        let path = navmesh.get_path_between_nodes(start_node, end_node);

        assert!(path.len() == 9);
        assert!(*path.first().unwrap() == start_node);
        assert!(*path.last().unwrap() == end_node);
    }
}
