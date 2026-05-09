pub struct Registers {
    pub(crate) a: u8,
    pub(crate) x: u8,
    pub(crate) y: u8
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            x: 0,
            y: 0
        }
    }
}