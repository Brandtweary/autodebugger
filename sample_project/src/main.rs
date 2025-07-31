fn main() {
    println!("Hello from the sample project!");
    
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    println!("Sum of {:?} is {}", numbers, sum);
    
    greet("Hector");
}

fn greet(name: &str) {
    println!("Hello, {}! Welcome to the autodebugger sample project.", name);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_greet() {
        greet("Test");
    }
}