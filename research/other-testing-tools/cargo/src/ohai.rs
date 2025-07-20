

pub fn ohai(a: &str) -> String {
    format!("ohai {}!", a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ohai() {
        assert_eq!(ohai("there"), "ohai there!");
    }
}