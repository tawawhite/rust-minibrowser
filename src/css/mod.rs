extern crate pom;
use pom::parser::{Parser,is_a,one_of,sym, none_of,seq};
use pom::char_class::alpha;
use std::str::{self, FromStr};
use self::pom::char_class::alphanum;
use std::fs::File;
use std::io::Read;



#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    pub(crate) rules: Vec<Rule>,
}
#[derive(Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}
#[derive(Debug, PartialEq)]
pub enum Selector {
    Simple(SimpleSelector)
}
#[derive(Debug, PartialEq)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}
#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub(crate) name: String,
    pub(crate) value: Value,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
    HexColor(String),
}

impl Value {
    pub fn to_px(&self) -> f32 {
        match *self {
            Value::Length(f, Unit::Px) => f,
            _ => 0.0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Unit {
    Px,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Color {
    pub(crate) r:u8,
    pub(crate) g:u8,
    pub(crate) b:u8,
    pub(crate) a:u8,
}

pub type Specificity = (usize, usize, usize);

impl Selector {
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a,b,c)
    }
}


fn space<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t\r\n").repeat(0..).discard()
}

fn number<'a>() -> Parser<'a, u8, f64> {
    let integer = one_of(b"123456789") - one_of(b"0123456789").repeat(0..) | sym(b'0');
    let frac = sym(b'.') + one_of(b"0123456789").repeat(1..);
    let exp = one_of(b"eE") + one_of(b"+-").opt() + one_of(b"0123456789").repeat(1..);
    let number = sym(b'-').opt() + integer + frac.opt() + exp.opt();
    number.collect().convert(str::from_utf8).convert(|s|f64::from_str(&s))
}

fn string<'a>() -> Parser<'a, u8, String> {
    let special_char = sym(b'\\') | sym(b'/') | sym(b'"')
        | sym(b'b').map(|_|b'\x08') | sym(b'f').map(|_|b'\x0C')
        | sym(b'n').map(|_|b'\n') | sym(b'r').map(|_|b'\r') | sym(b't').map(|_|b'\t');
    let escape_sequence = sym(b'\\') * special_char;
    let string = sym(b'"') * (none_of(b"\\\"") | escape_sequence).repeat(0..) - sym(b'"');
    string.convert(String::from_utf8)
}

fn v2s(v:&Vec<u8>) -> String {
    str::from_utf8(v).unwrap().to_string()
}

pub fn star(term:u8) -> bool {
    term == b'*'
}

fn alpha_string<'a>() -> Parser<'a, u8, String> {
    let r = is_a(alpha).repeat(1..);
    r.map(|str| String::from_utf8(str).unwrap())
}
fn alphanum_string<'a>() -> Parser<'a, u8, String> {
    let r = is_a(alphanum).repeat(1..);
    r.map(|str| String::from_utf8(str).unwrap())
}
fn star_string<'a>() -> Parser<'a, u8, String> {
    let r = sym(b'*');
    r.map(|str|{
        let mut s = String::new();
        s.push(char::from(str));
        s
    })
}
fn class_string<'a>() -> Parser<'a,u8,String> {
    let r = sym(b'.') + alphanum_string();
    r.map(|(dot,str)| {
        let mut s = String::from(str);
        s.insert(0,char::from(dot));
        s
    })
}

fn selector<'a>() -> Parser<'a, u8, Selector>{
    let r
        = space()
        + (class_string() | star_string() | alphanum_string())
    ;
    r.map(|(_,name)| {
        if name.starts_with(".") {
            Selector::Simple(SimpleSelector {
                tag_name: None,
                id: None,
                class: vec![name]
            })
        } else {
            Selector::Simple(SimpleSelector {
                tag_name: Some(name),
                id: None,
                class: vec![]
            })
        }
    })
}

#[test]
fn test_div_selector() {
    let input = br#"div"#;
    let result = selector().parse(input);
    println!("{:?}", result);
    assert_eq!(Selector::Simple(SimpleSelector{
        tag_name:Some("div".to_string()),
        id: None,
        class: vec![],
    }), result.unwrap())
}

#[test]
fn test_h3_selector() {
    let input = br#"h3"#;
    let result = selector().parse(input);
    println!("{:?}", result);
    assert_eq!(Selector::Simple(SimpleSelector{
        tag_name:Some("h3".to_string()),
        id: None,
        class: vec![],
    }), result.unwrap())
}

#[test]
fn test_class_selector() {
    let input = br#".cool"#;
    let result = selector().parse(input);
    println!("{:?}", result);
    assert_eq!(Selector::Simple(SimpleSelector{
        tag_name:None,
        id: None,
        class: vec![".cool".to_string()],
    }), result.unwrap())
}

#[test]
fn test_all_selector() {
    let input = br#"*"#;
    let result = selector().parse(input);
    println!("{:?}", result);
    assert_eq!(Selector::Simple(SimpleSelector{
        tag_name:Some("*".to_string()),
        id: None,
        class: vec![],
    }), result.unwrap())
}

fn identifier<'a>() -> Parser<'a, u8, String> {
    let r
        = space()
        + is_a(alpha)
        + (is_a(alphanum) | sym(b'-')).repeat(0..)
        ;
    r.map(|((_,uu),v)| {
        let mut vv = vec![uu];
        vv.extend(&v);
        return v2s(&vv)
    })
}
#[test]
fn test_identifier() {
    let input = br"bar";
    println!("{:?}",identifier().parse(input));
}

//if px, then turn Unit::px
fn unit<'a>() -> Parser<'a, u8, Unit> {
    seq(&br"px"[0..]).map(|_| Unit::Px)
}

#[test]
fn test_unit() {
    let input = br"px";
    println!("{:?}",unit().parse(input))
}

fn length_unit<'a>() -> Parser<'a, u8, Value> {
    let p = number() + unit();
    p.map(|(v,unit)| {
        Value::Length(v as f32,unit)
    })
}

#[test]
fn test_length_unit() {
    let input = br"3px";
    println!("{:?}",length_unit().parse(input))
}

fn hexcolor<'a>() -> Parser<'a, u8, Value> {
    let p = sym(b'#')
    * one_of(b"0123456789ABCDEFabcdef").repeat(6..7);
    p.map(|mut c| {
        // i32::from_str_radix(v2s(&c),16)
        c.insert(0,b'#');
        Value::HexColor(v2s(&c).to_lowercase())
    })
}

#[test]
fn test_hexcolor() {
    let input = br"#4455fF";
    let result = hexcolor().parse(input);
    println!("{:?}", result);
    assert_eq!( Value::HexColor("#4455FF".to_lowercase()), result.unwrap());
}

fn keyword<'a>() -> Parser<'a, u8, Value> {
    let r
        = space()
        + (is_a(alpha)).repeat(0..)
        ;
    r.map(|(_,c)| {
        Value::Keyword(String::from_utf8(c).unwrap())
    })
}

#[test]
fn test_keyword() {
    let input = br"black";
    println!("{:#?}",keyword().parse(input))
}

fn value<'a>() -> Parser<'a, u8, Value> {
    hexcolor() | length_unit() | keyword()
}

fn declaration<'a>() -> Parser<'a, u8, Declaration> {
    let r = space()
        + identifier()
        - (space() - sym(b':') - space())
        + value()
        - (space() - sym(b';') - space())
    ;
    r.map(|(((), name), value)| Declaration { name, value })
}

#[test]
fn test_prop_def() {
    let input = br#"border:black;"#;
    println!("{:?}", declaration().parse(input))
}
#[test]
fn test_prop_def2() {
    let input = b"border-color:black;";
    println!("{:?}", declaration().parse(input))
}
#[test]
fn test_prop_def3() {
    let input = b"border-width:1px;";
    println!("{:?}", declaration().parse(input))
}
#[test]
fn test_prop_def4() {
    let input = b"border-color:#ff00aa;";
    let result = declaration().parse(input);
    println!("{:?}", result);
    assert_eq!(Declaration {
        name: "border-color".to_string(),
        value: Value::HexColor("#ff00aa".to_lowercase())
    },result.unwrap());
    println!("{:?}", declaration().parse(input))
}

fn ws_sym<'a>(ch:u8) -> Parser<'a, u8,u8> {
    space() * sym(ch) - space()
}

fn rule<'a>() -> Parser<'a, u8, Rule> {
    let r
        = selector()
        - ws_sym(b'{')
        + declaration().repeat(0..)
        - ws_sym(b'}')
        ;
    r.map(|(sel, declarations)| Rule {
        selectors: vec![sel],
        declarations,
    })
}

#[test]
fn test_rule() {
    let input = b"div { border-width:1px; }";
    println!("{:#?}",rule().parse(input))
}
fn stylesheet<'a>() -> Parser<'a, u8, Stylesheet> {
    rule().repeat(0..).map(|rules| Stylesheet { rules })
}

#[test]
fn test_stylesheet() {
    let input = b"div { border-width:1px; } .cool { color: red; }";
    println!("{:#?}",stylesheet().parse(input))
}

#[test]
fn test_font_style() {
    let input = b"div { font-size: 18px; }";
    println!("{:#?}",stylesheet().parse(input))
}

pub fn load_stylesheet(filename:&str) -> Stylesheet {
    let mut file = File::open(filename).unwrap();
    let mut content:Vec<u8>= Vec::new();
    file.read_to_end(&mut content);
    return stylesheet().parse(content.as_slice()).unwrap();
}
pub fn parse_stylesheet(text:&str) -> Stylesheet {
    return stylesheet().parse(text.as_ref()).unwrap();
}

#[test]
fn test_file_load() {
    let mut file = File::open("tests/foo.css").unwrap();
    let mut content:Vec<u8>= Vec::new();
    file.read_to_end(&mut content);
    let parsed = stylesheet().parse(content.as_slice()).unwrap();
    println!("{:#?}", parsed);
    let ss = Stylesheet {
        rules: vec![
            Rule {
                selectors: vec![
                    Selector::Simple(SimpleSelector{
                        tag_name: Some(String::from("div")),
                        id: None,
                        class: vec![],
                    })
                ],
                declarations: vec![
                    Declaration {
                        name: "background-color".to_string(),
                        value: Value::Keyword("white".to_string()),
                    },
                    Declaration {
                        name: "border-color".to_string(),
                        value: Value::Keyword("red".to_string()),
                    },
                    Declaration {
                        name: "border-width".to_string(),
                        value: Value::Length(1.0,Unit::Px),
                    },
                    Declaration {
                        name: "color".to_string(),
                        value: Value::Keyword("black".to_string()),
                    },
                ],
            },
            Rule {
                selectors: vec![
                    Selector::Simple(SimpleSelector{
                        tag_name: None,
                        id: None,
                        class: vec![String::from(".cool")]
                    })
                ],
                declarations: vec![
                    Declaration {
                        name: "color".to_string(),
                        value: Value::Keyword("green".to_string()),
                    },
                ],
            }
        ]
    };
    assert_eq!(ss,parsed)
}
