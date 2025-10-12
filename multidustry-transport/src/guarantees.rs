pub enum Guarantees {
    Reliable,
    Unreliable,
}

impl Default for Guarantees {
    fn default() -> Self {
        Guarantees::Reliable
    }
}
