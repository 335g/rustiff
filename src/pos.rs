
#[derive(Debug)]
#[non_exhaustive]
pub enum Position {
    Unknown,
    IFD(u64),
    Data(u64),
}