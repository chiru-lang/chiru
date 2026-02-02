#[derive(Debug)]
pub enum AstNode {
    Function { name: String, body: Vec<AstNode> },
    Unsafe { body: Vec<AstNode> },

    Region { kind: String, name: String },
    Lifetime { name: String, scope: String },
    Let { name: String, region: String },

    Capability {
        kind: String,
        value: String,
        lifetime: String,
    },

    Drop { value: String },
    Assume { text: String },
}
