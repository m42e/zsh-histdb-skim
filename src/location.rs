use enum_map::Enum;

#[derive(PartialEq, Enum, Copy, Clone)]
pub enum Location {
    Session,
    Directory,
    Machine,
    Everywhere,
}
