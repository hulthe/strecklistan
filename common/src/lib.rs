#[macro_use]
extern crate getset;

pub mod models;

pub use models::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
