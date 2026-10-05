use arrayvec::ArrayString;

#[derive(Debug, Clone, PartialEq)]
pub struct Airway {
    pub identifier: ArrayString<7>,
}
