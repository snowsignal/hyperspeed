#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Input {
    Click { x: u32, y: u32 },
    Key(String)
}