extern crate rand;

mod network_sim;
use network_sim::Network;


#[cfg(not(test))]
fn check_unique_coord() {
    let mut rng = rand::thread_rng();
    let l: u32 = 20;
    let n: usize = ((2 as u64).pow(l)) as usize;
    let num_neighbours: usize = (1.5 * (n as f64).ln()) as usize;
    let num_landmarks: usize = (l*l) as usize;
    let num_iter_coords = (1.5 * ((l as f64) / (num_neighbours as f64).ln())) as u32;

    println!("n = {}",n);
    println!("num_neighbours = {}",num_neighbours);
    println!("num_landmarks = {}",num_landmarks);
    println!("num_iters_coords = {}",num_iter_coords);

    let mut net = Network::new()
        .build_network(n,num_neighbours,&mut rng)
        .choose_landmarks(num_landmarks,&mut rng);

    for i in 0..num_iter_coords {
        net.iter_coords();
        println!("Iter number {}",i);
    }

    
    let is_unique = net.is_coord_unique();
    println!("is_unique = {}",is_unique);

    // net.print_some_coords(10);

}



#[cfg(not(test))]
fn main() {
    // let net = Network::new();
    // let mut rng = rand::thread_rng();
    check_unique_coord();
}
