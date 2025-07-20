mod sum;
mod ohai;

fn main() {
    println!("{}", ohai::ohai(&sum::sum(1, 2).to_string()));
}   
