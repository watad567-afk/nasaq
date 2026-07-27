//! Server-side rendering for Nasaq Web components with initial state evaluation.

use std::collections::HashMap;

use nasaq_ast::{AttrValue, ComponentDecl, Expr, HtmlAttr, Item, Literal, Program, ViewNode};
use nasaq_syntax::Spanned;

pub struct SsrOptions {
    pub mount_selector: String,
    pub hydrate: bool,
}

impl Default for SsrOptions {
    fn default() -> Self {
        Self {
            mount_selector: "#app".into(),
            hydrate: true,
        }
    }
}

pub fn render_component_html(component: &ComponentDecl) -> String {
    let mut registry = HashMap::new();
    registry.insert(component.name.node.clone(), component);
    let ctx = RenderContext::new(component, &registry);
    let mut html = String::new();
    if let Some(view) = &component.view {
        ctx.render_nodes(&view.node.nodes, &mut html);
    }
    html
}

pub fn render_component_html_from_program(program: &Program, name: &str) -> Option<String> {
    let registry = collect_components(program);
    let component = registry.get(name)?;
    let ctx = RenderContext::new(component, &registry);
    let mut html = String::new();
    if let Some(view) = &component.view {
        ctx.render_nodes(&view.node.nodes, &mut html);
    }
    Some(html)
}

fn collect_components(program: &Program) -> HashMap<String, &ComponentDecl> {
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

pub fn render_ssr_document(
    title: &str,
    body: &str,
    module_name: &str,
    options: &SsrOptions,
) -> String {
    let hydrate = if options.hydrate { "hydrate" } else { "mount" };
    format!(
        r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title}</title>
  <style>
    body {{ font-family: system-ui, sans-serif; margin: 2rem; background: #0b1020; color: #e5e7eb; }}
    .counter {{ display: grid; gap: 1rem; max-width: 24rem; }}
    button {{ padding: 0.5rem 1rem; cursor: pointer; border-radius: 8px; border: 0; background: #2563eb; color: white; }}
  </style>
</head>
<body>
  <div id="app">{body}</div>
  <script type="module">
    import {{ {hydrate}Component }} from './runtime/{dom}';
    import {{ Counter }} from './{module_name}.{ext}';
    {hydrate}Component(Counter, '{selector}');
  </script>
</body>
</html>
"#,
        title = title,
        body = body,
        module_name = module_name,
        hydrate = hydrate,
        selector = options.mount_selector,
        dom = nasaq_syntax::with_runtime_ext("dom"),
        ext = nasaq_syntax::OUTPUT,
    )
}

struct RenderContext<'a> {
    states: HashMap<String, i32>,
    params: HashMap<String, i32>,
    click_index: std::cell::Cell<usize>,
    components: &'a HashMap<String, &'a ComponentDecl>,
}

impl<'a> RenderContext<'a> {
    fn new(component: &ComponentDecl, components: &'a HashMap<String, &'a ComponentDecl>) -> Self {
        let mut params = HashMap::new();
        for p in &component.params {
            if let Some(default) = &p.node.default {
                if let Some(v) = eval_int(&default.node, &params, &HashMap::new()) {
                    params.insert(p.node.name.node.clone(), v);
                }
            }
        }
        let mut states = HashMap::new();
        for state in &component.states {
            if let Some(v) = eval_int(&state.node.init.node, &params, &states) {
                states.insert(state.node.name.node.clone(), v);
            }
        }
        Self {
            states,
            params,
            click_index: std::cell::Cell::new(0),
            components,
        }
    }

    fn render_nodes(&self, nodes: &[Spanned<ViewNode>], out: &mut String) {
        for node in nodes {
            self.render_node(node, out);
        }
    }

    fn render_node(&self, node: &Spanned<ViewNode>, out: &mut String) {
        match &node.node {
            ViewNode::Text(text) => out.push_str(&text.node),
            ViewNode::Interpolation(expr) => {
                if let Expr::Ident(name) = &expr.node {
                    if self.states.contains_key(&name.node) {
                        let value = self.states.get(&name.node).copied().unwrap_or(0);
                        out.push_str(&format!(
                            "<span data-nasaq-signal=\"{}\">{value}</span>",
                            name.node
                        ));
                        return;
                    }
                }
                let value = eval_int(&expr.node, &self.params, &self.states)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                out.push_str(&value);
            }
            ViewNode::Element(el) => {
                if is_component_tag(&el.tag.node) {
                    self.render_component_el(el, out);
                    return;
                }
                out.push('<');
                out.push_str(&el.tag.node);
                for attr in &el.attrs {
                    self.render_attr(attr, out);
                }
                out.push('>');
                self.render_nodes(&el.children, out);
                out.push_str("</");
                out.push_str(&el.tag.node);
                out.push('>');
            }
        }
    }

    fn render_component_el(&self, el: &nasaq_ast::HtmlElement, out: &mut String) {
        let Some(comp) = self.components.get(&el.tag.node) else {
            out.push_str(&format!("<!-- unknown component {} -->", el.tag.node));
            return;
        };
        let mut params = HashMap::new();
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
                    if let AttrValue::Expr(expr) = &value.node {
                        if let Some(v) = eval_int(&expr.node, &self.params, &self.states) {
                            params.insert(param_name.clone(), v);
                        }
                    }
                }
            } else if let Some(default) = &param.node.default {
                if let Some(v) = eval_int(&default.node, &params, &HashMap::new()) {
                    params.insert(param_name.clone(), v);
                }
            }
        }
        let mut states = HashMap::new();
        for state in &comp.states {
            if let Some(v) = eval_int(&state.node.init.node, &params, &states) {
                states.insert(state.node.name.node.clone(), v);
            }
        }
        let child = RenderContext {
            states,
            params,
            click_index: self.click_index.clone(),
            components: self.components,
        };
        if let Some(view) = &comp.view {
            child.render_nodes(&view.node.nodes, out);
        }
    }

    fn render_attr(&self, attr: &Spanned<HtmlAttr>, out: &mut String) {
        match &attr.node {
            HtmlAttr::Attribute { name, value } => {
                out.push(' ');
                out.push_str(&name.node);
                out.push('=');
                out.push('"');
                match &value.node {
                    AttrValue::String(s) => out.push_str(s),
                    AttrValue::Expr(expr) => {
                        out.push_str(
                            &eval_int(&expr.node, &self.params, &self.states)
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                        );
                    }
                }
                out.push('"');
            }
            HtmlAttr::Event { event, .. } => {
                let idx = self.click_index.get();
                self.click_index.set(idx + 1);
                out.push_str(&format!(
                    " data-nasaq-click=\"{idx}\" data-nasaq-event=\"{}\"",
                    event.node
                ));
            }
        }
    }
}

fn eval_int(expr: &Expr, params: &HashMap<String, i32>, states: &HashMap<String, i32>) -> Option<i32> {
    match expr {
        Expr::Literal(lit) => match &lit.node {
            Literal::Int(v) => Some(*v as i32),
            _ => None,
        },
        Expr::Ident(name) => states
            .get(&name.node)
            .or_else(|| params.get(&name.node))
            .copied(),
        Expr::Binary { op, left, right, .. } => {
            let l = eval_int(&left.node, params, states)?;
            let r = eval_int(&right.node, params, states)?;
            Some(match op {
                nasaq_ast::BinOp::Add => l + r,
                nasaq_ast::BinOp::Sub => l - r,
                nasaq_ast::BinOp::Mul => l * r,
                nasaq_ast::BinOp::Div => if r == 0 { 0 } else { l / r },
                nasaq_ast::BinOp::Mod => if r == 0 { 0 } else { l % r },
                _ => l,
            })
        }
        Expr::Unary { op, expr, .. } => {
            let v = eval_int(&expr.node, params, states)?;
            Some(if matches!(op, nasaq_ast::UnaryOp::Neg) {
                -v
            } else {
                v
            })
        }
        _ => None,
    }
}
