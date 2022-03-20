use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar);

pub use grammar::MacroParser;

#[derive(Clone, Debug)]
pub struct Macro {
    pub patterns: Vec<Pattern>,
    pub effect: Effect,
}

#[derive(Clone, Debug)]
pub struct Pattern {
    pub selector: Selector,
    pub where_clause: Option<Where>,
}

#[derive(Clone, Debug)]
pub struct Selector {
    pub tag: Tag,
    pub id: Id,
}

#[derive(Clone, Debug)]
pub enum Tag {
    Item,
    Bundle,
}

#[derive(Clone, Debug)]
pub enum Id {
    Any,
    Is(String),
}

#[derive(Clone, Debug)]
pub struct Where {
    pub field: Field,
    pub operator: Op,
    pub value: f64,
}

#[derive(Clone, Debug)]
pub enum Field {
    Price,
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    GrEq,
    GrTh,
    LeEq,
    LeTh,
    Eq,
    NotEq,
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub tag: Tag,
    pub name: String,
}
