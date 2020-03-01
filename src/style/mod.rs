use crate::dom::{Node, ElementData, load_doc};
use crate::css::{Selector, SimpleSelector, Rule, Stylesheet, Specificity, Value, Color, load_stylesheet};
use std::collections::HashMap;
//use serde_json::Value;
use crate::css::Selector::Simple;
use crate::dom::NodeType::{Element, Text};
use crate::css::Value::{Keyword, ColorValue, Length};
use crate::layout::Display;
use crate::render::BLACK;

type PropertyMap = HashMap<String, Value>;

#[derive(Debug, PartialEq)]
pub struct StyledNode<'a> {
    pub(crate) node: &'a Node,
    pub specified_values: PropertyMap,
    pub(crate) children: Vec<StyledNode<'a>>,
}

impl StyledNode<'_> {
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).map(|v| v.clone())
    }
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }

    pub fn color(&self, name: &str) -> Color {
        match self.value(name) {
            Some(ColorValue(c)) => c,
            _ => BLACK,
        }
    }
    pub fn insets(&self, name: &str) -> f32 {
        match self.value(name) {
            Some(Length(v,_unit)) => v,
            _ => 0.0,
        }
    }
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector)
    }
}


fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    //return false for mis-matches
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }
    let elem_classes = elem.classes();
    if selector.class.iter().any(|class| !elem_classes.contains(&**class)) {
        return false
    }
    //no non-matching selectors found, so it must be true
    return true;
}

type MatchedRule<'a> = (Specificity, &'a Rule);

// return rule that matches, if any.
fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors.iter()
        .find(|selector| matches(elem, selector))
        .map(|selector| (selector.specificity(), rule))
}

//find all matching rules for an element
fn matching_rules<'a>(elem: &ElementData, stylesheeet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    stylesheeet.rules.iter().filter_map(|rule| match_rule(elem,rule)).collect()
}

// get all values set by all rules
fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values:HashMap<String,Value> = HashMap::new();
    let mut rules = matching_rules(elem,stylesheet);

    //sort rules by specificity
    rules.sort_by(|&(a,_),&(b,_)| a.cmp(&b));
    for (_,rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    return values;
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            Element(ref elem) => specified_values(elem, stylesheet),
            Text(_) => HashMap::new(),
        },
        children: root.children.iter().map(|child| style_tree(child, stylesheet)).collect()
    }
}

#[test]
fn test_style_tree() {
    let doc = load_doc("tests/test1.html");
    let stylesheet = load_stylesheet("tests/foo.css");
    let snode = style_tree(&doc,&stylesheet);
    println!("final snode is {:#?}",snode)
}

