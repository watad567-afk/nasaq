use std::collections::{HashMap, HashSet};

use nasaq_ast::{
    AttrValue, ComponentDecl, Expr, HtmlAttr, HtmlElement, Item, Program, ViewNode,
};
use nasaq_syntax::Spanned;

pub struct CompiledComponent {
    pub name: String,
    pub js: String,
}

pub fn compile_program_components(program: &Program) -> String {
    let registry = collect_components(program);
    let mut out = String::new();
    for comp in registry.values() {
        out.push_str(&compile_component(comp, &registry).js);
    }
    out
}

pub fn compile_component(
    component: &ComponentDecl,
    registry: &HashMap<String, &ComponentDecl>,
) -> CompiledComponent {
    let mut gen = ComponentCodegen::new(&component.name.node, registry);
    for state in &component.states {
        gen.states.insert(state.node.name.node.clone());
    }
    gen.emit_component(component);
    CompiledComponent {
        name: component.name.node.clone(),
        js: gen.output,
    }
}

fn collect_components<'a>(program: &'a Program) -> HashMap<String, &'a ComponentDecl> {
    let mut map = HashMap::new();
    for item in &program.items {
        if let Some(comp) = component_from_item(&item.node) {
            map.insert(comp.name.node.clone(), comp);
        }
    }
    map
}

fn component_from_item(item: &Item) -> Option<&ComponentDecl> {
    match item {
        Item::Component(c) => Some(c),
        Item::Export(inner) => component_from_item(&inner.node),
        _ => None,
    }
}

fn is_component_tag(tag: &str) -> bool {
    tag.chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
}

struct ComponentCodegen<'a> {
    name: String,
    output: String,
    indent: usize,
    states: HashSet<String>,
    click_handlers: Vec<String>,
    components: &'a HashMap<String, &'a ComponentDecl>,
}

impl<'a> ComponentCodegen<'a> {
    fn new(name: &str, components: &'a HashMap<String, &'a ComponentDecl>) -> Self {
        Self {
            name: name.to_string(),
            output: String::new(),
            indent: 0,
            states: HashSet::new(),
            click_handlers: Vec::new(),
            components,
        }
    }

    fn emit_component(&mut self, comp: &ComponentDecl) {
        let params: Vec<_> = comp
            .params
            .iter()
            .map(|p| {
                let default = p
                    .node
                    .default
                    .as_ref()
                    .map(|d| self.expr_to_js(&d.node))
                    .unwrap_or_else(|| "undefined".into());
                format!("{} = {}", p.node.name.node, default)
            })
            .collect();
        self.writeln(&format!(
            "export function {}({}) {{",
            comp.name.node,
            params.join(", ")
        ));
        self.indent += 1;

        for state in &comp.states {
            let init = self.expr_to_js(&state.node.init.node);
            self.writeln(&format!(
                "const {} = createSignal({});",
                state.node.name.node, init
            ));
        }

        if let Some(style) = &comp.style {
            if style.node.scoped {
                self.writeln(&format!(
                    "const __style = document.createElement('style'); __style.textContent = {}; document.head.appendChild(__style);",
                    json_string(&style.node.css.node)
                ));
            }
        }

        self.writeln("function mount(root) {");
        self.indent += 1;
        if let Some(view) = &comp.view {
            let root_var = self.emit_view_nodes(&view.node.nodes, "root");
            let _ = root_var;
        }
        self.indent -= 1;
        self.writeln("}");

        let handlers = if self.click_handlers.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "[{}]",
                self.click_handlers
                    .iter()
                    .map(|h| format!("(event) => {{ {h}; }}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let signal_map: Vec<_> = self
            .states
            .iter()
            .map(|s| format!("{s}: {s}"))
            .collect();
        self.writeln(&format!("const clickHandlers = {handlers};"));
        self.writeln(&format!(
            "const signals = {{ {} }};",
            signal_map.join(", ")
        ));

        self.writeln("function hydrate(root) {");
        self.indent += 1;
        self.writeln("root.querySelectorAll('[data-nasaq-signal]').forEach((el) => {");
        self.indent += 1;
        self.writeln("const name = el.getAttribute('data-nasaq-signal');");
        self.writeln("const signal = signals[name];");
        self.writeln("if (!signal) return;");
        self.writeln("effect(() => { el.textContent = String(signal.get()); });");
        self.indent -= 1;
        self.writeln("});");
        self.writeln("root.querySelectorAll('[data-nasaq-click]').forEach((el) => {");
        self.indent += 1;
        self.writeln("const idx = Number(el.getAttribute('data-nasaq-click'));");
        self.writeln("const eventName = el.getAttribute('data-nasaq-event') || 'click';");
        self.writeln("const handler = clickHandlers[idx];");
        self.writeln("if (handler) el.addEventListener(eventName, handler);");
        self.indent -= 1;
        self.writeln("});");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("return { mount, hydrate, clickHandlers, signals };");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");
    }

    fn emit_view_nodes(&mut self, nodes: &[Spanned<ViewNode>], parent: &str) -> String {
        for node in nodes {
            self.emit_view_node(node, parent);
        }
        parent.to_string()
    }

    fn emit_view_node(&mut self, node: &Spanned<ViewNode>, parent: &str) {
        match &node.node {
            ViewNode::Text(text) => {
                self.writeln(&format!(
                    "{{ const __t = document.createTextNode({}); {}.appendChild(__t); }}",
                    json_string(&text.node),
                    parent
                ));
            }
            ViewNode::Interpolation(expr) => {
                let signal_name = match &expr.node {
                    Expr::Ident(name) if self.states.contains(&name.node) => Some(name.node.clone()),
                    _ => None,
                };
                let id = fresh_id("span");
                if let Some(name) = signal_name {
                    self.writeln(&format!(
                        "{{ const {id} = document.createElement('span'); {id}.setAttribute('data-nasaq-signal', {sig}); {id}.textContent = String({expr}); {parent}.appendChild({id}); effect(() => {{ {id}.textContent = String({expr}); }}); }}",
                        id = id,
                        sig = json_string(&name),
                        parent = parent,
                        expr = self.expr_to_js(&expr.node)
                    ));
                } else {
                    self.writeln(&format!(
                        "{{ const {id} = document.createElement('span'); {id}.textContent = String({expr}); {parent}.appendChild({id}); effect(() => {{ {id}.textContent = String({expr}); }}); }}",
                        id = id,
                        parent = parent,
                        expr = self.expr_to_js(&expr.node)
                    ));
                }
            }
            ViewNode::Element(el) => {
                if is_component_tag(&el.tag.node) {
                    self.emit_component_usage(el, parent);
                    return;
                }
                let var = fresh_id("el");
                self.writeln(&format!(
                    "const {var} = document.createElement({tag});",
                    var = var,
                    tag = json_string(&el.tag.node)
                ));
                for attr in &el.attrs {
                    self.emit_attr(&var, attr);
                }
                self.writeln(&format!("{}.appendChild({});", parent, var));
                for child in &el.children {
                    self.emit_view_node(child, &var);
                }
            }
        }
    }

    fn emit_component_usage(&mut self, el: &HtmlElement, parent: &str) {
        let name = &el.tag.node;
        let args = if let Some(comp) = self.components.get(name) {
            component_call_args(el, comp, self)
        } else {
            String::new()
        };
        let host = fresh_id("host");
        self.writeln(&format!(
            "{{ const {host} = document.createElement('div'); {parent}.appendChild({host}); {name}({args}).mount({host}); }}"
        ));
    }

    fn emit_attr(&mut self, el: &str, attr: &Spanned<HtmlAttr>) {
        match &attr.node {
            HtmlAttr::Attribute { name, value } => match &value.node {
                AttrValue::String(s) => {
                    self.writeln(&format!(
                        "{}.setAttribute({}, {});",
                        el,
                        json_string(&name.node),
                        json_string(s)
                    ));
                }
                AttrValue::Expr(expr) => {
                    self.writeln(&format!(
                        "effect(() => {{ {}.setAttribute({}, String({})); }});",
                        el,
                        json_string(&name.node),
                        self.expr_to_js(&expr.node)
                    ));
                }
            },
            HtmlAttr::Event { event, handler } => {
                let handler_js = self.expr_to_js(&handler.node);
                let idx = self.click_handlers.len();
                self.click_handlers.push(handler_js.clone());
                self.writeln(&format!(
                    "{}.setAttribute('data-nasaq-click', '{idx}'); {}.setAttribute('data-nasaq-event', {});",
                    el,
                    el,
                    json_string(&event.node)
                ));
                self.writeln(&format!(
                    "{}.addEventListener({}, (event) => {{ {}; }});",
                    el,
                    json_string(&event.node),
                    handler_js
                ));
            }
        }
    }

    fn expr_to_js(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => match &lit.node {
                nasaq_ast::Literal::Int(v) => v.to_string(),
                nasaq_ast::Literal::Float(v) => v.clone(),
                nasaq_ast::Literal::String(v) => json_string(v),
                nasaq_ast::Literal::Char(c) => json_string(&c.to_string()),
                nasaq_ast::Literal::Bool(b) => b.to_string(),
            },
            Expr::Ident(name) => {
                if self.states.contains(&name.node) {
                    format!("{}.get()", name.node)
                } else {
                    name.node.clone()
                }
            }
            Expr::Binary { op, left, right, .. } => format!(
                "({} {} {})",
                self.expr_to_js(&left.node),
                binop(op),
                self.expr_to_js(&right.node)
            ),
            Expr::Unary { op, expr, .. } => format!(
                "({}{})",
                unary(op),
                self.expr_to_js(&expr.node)
            ),
            Expr::Assign { target, value, .. } => {
                if let Expr::Ident(name) = &target.node {
                    if self.states.contains(&name.node) {
                        return format!(
                            "{}.set({})",
                            name.node,
                            self.expr_to_js(&value.node)
                        );
                    }
                }
                "undefined".into()
            }
            Expr::Call { callee, args, .. } => {
                let callee = self.expr_to_js(&callee.node);
                let args: Vec<_> = args.iter().map(|a| self.expr_to_js(&a.node)).collect();
                format!("{callee}({})", args.join(", "))
            }
            _ => "undefined".to_string(),
        }
    }

    fn writeln(&mut self, s: &str) {
        self.output.push_str(&"    ".repeat(self.indent));
        self.output.push_str(s);
        self.output.push('\n');
    }
}

fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('`', "\\`")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn fresh_id(prefix: &str) -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("_{prefix}{n}")
}

fn binop(op: &nasaq_ast::BinOp) -> &'static str {
    match op {
        nasaq_ast::BinOp::Add => "+",
        nasaq_ast::BinOp::Sub => "-",
        nasaq_ast::BinOp::Mul => "*",
        nasaq_ast::BinOp::Div => "/",
        nasaq_ast::BinOp::Mod => "%",
        nasaq_ast::BinOp::Eq => "===",
        nasaq_ast::BinOp::Ne => "!==",
        nasaq_ast::BinOp::Lt => "<",
        nasaq_ast::BinOp::Le => "<=",
        nasaq_ast::BinOp::Gt => ">",
        nasaq_ast::BinOp::Ge => ">=",
        nasaq_ast::BinOp::And => "&&",
        nasaq_ast::BinOp::Or => "||",
    }
}

fn unary(op: &nasaq_ast::UnaryOp) -> &'static str {
    match op {
        nasaq_ast::UnaryOp::Neg => "-",
        nasaq_ast::UnaryOp::Not => "!",
    }
}

fn json_string(s: &str) -> String {
    format!("\"{}\"", escape_js(s))
}

fn component_call_args(el: &HtmlElement, comp: &ComponentDecl, gen: &ComponentCodegen<'_>) -> String {
    let mut args = Vec::new();
    for param in &comp.params {
        let param_name = &param.node.name.node;
        let attr = el.attrs.iter().find(|a| {
            matches!(
                &a.node,
                HtmlAttr::Attribute { name, .. } if name.node == *param_name
            )
        });
        if let Some(attr) = attr {
            if let HtmlAttr::Attribute { value, .. } = &attr.node {
                match &value.node {
                    AttrValue::String(s) => args.push(json_string(s)),
                    AttrValue::Expr(expr) => args.push(gen.expr_to_js(&expr.node)),
                }
            }
        } else if let Some(default) = &param.node.default {
            args.push(gen.expr_to_js(&default.node));
        } else {
            args.push("undefined".into());
        }
    }
    args.join(", ")
}

pub fn compile_item(item: &Item, registry: &HashMap<String, &ComponentDecl>) -> Option<CompiledComponent> {
    match item {
        Item::Component(c) => Some(compile_component(c, registry)),
        Item::Export(inner) => compile_item(&inner.node, registry),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nasaq_parser::parse_program;

    #[test]
    fn compiles_counter_component() {
        let src = r#"
            export component Counter(start: Int = 0) {
                state count: Int = start
                view {
                    <button on:click={ count = count + 1 }>
                        Increase
                    </button>
                }
            }
        "#;
        let parsed = parse_program(src);
        assert!(!parsed.diagnostics.has_errors(), "{:?}", parsed.diagnostics.diagnostics);
        let program = parsed.program.unwrap();
        let comp = match &program.items[0].node {
            Item::Component(c) => c,
            _ => panic!("expected component"),
        };
        let registry = collect_components(&program);
        let out = compile_component(comp, &registry);
        assert!(out.js.contains("createSignal"));
        assert!(out.js.contains("addEventListener"));
    }

    #[test]
    fn compiles_nested_component_usage() {
        let src = r#"
            export component Counter(start: Int = 0) {
                state count: Int = start
                view { <span>{ count }</span> }
            }
            export component App() {
                view { <Counter start={5} /> }
            }
        "#;
        let parsed = parse_program(src);
        assert!(!parsed.diagnostics.has_errors(), "{:?}", parsed.diagnostics.diagnostics);
        let program = parsed.program.unwrap();
        let out = compile_program_components(&program);
        assert!(out.contains("Counter(5).mount"));
    }
}
