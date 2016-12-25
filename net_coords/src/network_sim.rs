extern crate rand;

use std::f64;
use std::collections::HashSet;

// use self::rand;
use self::rand::Rng;
use self::rand::distributions::{IndependentSample, Range};

pub struct Network {
    n: usize,
    neighbours: Vec<HashSet<usize>>,
    landmarks: Vec<usize>,
    coords: Vec<Vec<usize>>,
}


/// Randomly choose k distinct numbers from the range [0,n) 
fn choose_k_nums<R: Rng>(k:usize, n:usize, rng: &mut R) -> HashSet<usize> {
    
    let mut res_set = HashSet::new();
    let rand_range: Range<usize> = Range::new(0,n);
    for _ in 0..k {
        let mut x = rand_range.ind_sample(rng);
        while res_set.contains(&x) {
            x = rand_range.ind_sample(rng);
        }
        res_set.insert(x);
    }
    res_set
}

/// Convert network coordinate to chord value in [0,1) 
/// by projection to a plane.
pub fn coord_to_ring(coord: &Vec<usize>) -> f64 {
    let fcoord: Vec<f64> = coord.iter().map(|&a| a as f64).collect();

    let k: f64 = fcoord.len() as f64;
    let S_a:f64 = fcoord.iter().sum();
    let normalize = |a| (a/S_a) - (1.0/k);
    let L_a: f64 = fcoord.iter().
        map(|&a| normalize(a).powi(2) as f64).sum::<f64>().sqrt();

    let numerator: f64 = 
        normalize(fcoord[0]) - 
            (1.0/(k-1.0)) * fcoord.iter().skip(1).map(|&a| normalize(a)).sum::<f64>();

    let denominator: f64 = L_a * ((k/(k-1.0))).sqrt();

    (numerator/denominator).acos() / (f64::consts::PI)
}

impl Network {
    pub fn new() -> Self {
        Network {
            n: 0, 
            neighbours: Vec::new(),
            landmarks: Vec::new(),
            coords: Vec::new(),
        }
    }

    pub fn build_network<R: Rng>(
        mut self, n: usize, num_neighbours: usize, rng: &mut R) -> Self {
        self.neighbours.clear();
        self.n = n;
        for _ in 0 .. n {
            self.neighbours.push(HashSet::new());
        }

        let rand_range: Range<usize> = Range::new(0,self.n);

        // Connect node v to about num_neighbours other nodes:
        for v in 0 .. self.n {
            for _ in 0 .. num_neighbours {
                let u = rand_range.ind_sample(rng);
                if u == v {
                    // Avoid self loops
                    continue
                }
                if self.neighbours[v].contains(&u) {
                    // Already has this edge.
                    continue
                }
                // Add edge:
                self.neighbours[v].insert(u);
                self.neighbours[u].insert(v);
            }
        }
        self
    }

    pub fn choose_landmarks<R: Rng> (mut self,num_landmarks: usize, rng: &mut R) 
        -> Self {

        self.landmarks = choose_k_nums(num_landmarks, self.n, rng)
            .into_iter().collect();

        self
    }

    /// Every node asks neighbours about distance to landmarks and 
    /// updates his own distances accordingly.
    /// Returns true if anything in the coords state has changed.
    pub fn iter_coords(&self, work_coords: &mut Vec<Vec<Option<usize>>>) -> bool {
        let mut has_changed = false;
        for v in 0..self.n {
            for &nei in self.neighbours[v].iter() {
                for c in 0..work_coords[nei].len() {
                    let dist = work_coords[nei][c];
                    if dist.is_none() {
                        continue
                    }
                    let cdist = dist.unwrap() + 1;
                    if work_coords[v][c].is_none() {
                        work_coords[v][c] = Some(cdist);
                        has_changed = true;
                        continue
                    }
                    if work_coords[v][c].unwrap() > cdist {
                        work_coords[v][c] = Some(cdist);
                        has_changed = true;
                    }
                }
            }
        }
        has_changed
    }

    pub fn converge_coords(&mut self) -> bool {
        let mut work_coords: Vec<Vec<Option<usize>>> = Vec::new();

        // Initialize coordinates:
        for v in 0..self.n {
            let mut v_coords = Vec::new();
            for &l in self.landmarks.iter() {
                if v != l {
                    v_coords.push(None)
                } else {
                    v_coords.push(Some(0))
                }
            }
            work_coords.push(v_coords);
        }

        let mut has_changed = true;
        while has_changed {
            has_changed = self.iter_coords(&mut work_coords);
            println!("Iter");
        }

        let is_disconnected: bool = 
            work_coords.iter().any(|coord| 
                   coord.iter().any(|&c_opt| c_opt.is_none()));

        if is_disconnected {
            return false;
        }

        self.coords = 
            work_coords.into_iter().map(|coord_opt| 
                coord_opt.into_iter().map(|c_opt| c_opt.unwrap()).collect::<Vec<_>>())
                .collect::<Vec<_>>();

        true
    }


    /// Check if the coordinates system is unique
    pub fn is_coord_unique(&self) -> bool {
        let mut coord_set = HashSet::new();
        for coord in self.coords.iter() {
            if coord_set.contains(coord) {
                return false
            }
            coord_set.insert(coord);
        }
        true
    }

    /// Get the cord id for some node v
    pub fn get_chord_id(&self, v:usize) {

    }

    /// Print some coordinates
    pub fn print_some_coords(&self,amount: u32) {
        for v in 0..amount {
            println!("{}", coord_to_ring(&self.coords[v as usize]));
            println!("{:?}", self.coords[v as usize]);
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut rng = rand::thread_rng();
        let mut net = Network::new()
            .build_network(40,3,&mut rng)
            .choose_landmarks(4,&mut rng);

        net.converge_coords();
        net.is_coord_unique();
    }

    #[test]
    fn test_choose_k_nums() {
        let mut rng = rand::thread_rng();
        let knums = choose_k_nums(3,100,&mut rng);
        assert!(knums.len() == 3);
        for x in knums.into_iter() {
            assert!(x < 100);
        }
    }

    #[test]
    fn test_coord_to_ring() {
        coord_to_ring(&vec![5,6,1,4]);
        coord_to_ring(&vec![5,6,1,5,4]);
    }

    #[test]
    fn test_hashset_vec() {
        let mut my_set : HashSet<Vec<usize>> = HashSet::new();
        my_set.insert(vec![1,2,3]);
        assert!(my_set.contains(&vec![1,2,3]));
        assert!(!my_set.contains(&vec![1,2,4]));
    }
}
