#[derive(Default)]
pub struct DebugResources {
    pub string_name: String,
    pub f32_number: f32,
    pub bool_number: bool,
    pub enum_variable: Positions,
}

#[derive(Default, PartialEq)]
pub enum Positions {
    #[default]
    First,
    Second,
    Third,
}
