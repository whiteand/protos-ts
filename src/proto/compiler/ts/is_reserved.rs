//

pub(super) fn is_reserved(name: &str) -> bool {
    match name {
        "do" | "if" | "in" | "for" | "let" | "new" | "try" | "var" | "case" | "else" | "enum"
        | "eval" | "false" | "null" | "this" | "true" | "void" | "with" | "break" | "catch"
        | "class" | "const" | "super" | "throw" | "while" | "yield" | "delete" | "export"
        | "import" | "public" | "return" | "static" | "switch" | "typeof" | "default"
        | "extends" | "finally" | "package" | "private" | "continue" | "debugger" | "function"
        | "arguments" | "interface" | "protected" | "implements" | "instanceof" => true,
        _ => false,
    }
}
