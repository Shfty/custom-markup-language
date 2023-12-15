use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

trait ToBBCode {
    fn to_bbcode(self) -> String;
}

impl ToBBCode for String {
    fn to_bbcode(self) -> String {
        self
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TextStyle {
    color: Option<String>,
    size: Option<usize>,
    bold: bool,
    italic: bool,
}

impl TextStyle {
    pub fn color(color: String) -> Self {
        Self {
            color: Some(color),
            ..Default::default()
        }
    }

    pub fn size(size: usize) -> Self {
        Self {
            size: Some(size),
            ..Default::default()
        }
    }

    pub fn bold() -> Self {
        Self {
            bold: true,
            ..Default::default()
        }
    }

    pub fn italic() -> Self {
        Self {
            italic: true,
            ..Default::default()
        }
    }
}

impl std::ops::BitOr for TextStyle {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            color: rhs.color.or(self.color),
            size: rhs.size.or(self.size),
            bold: rhs.bold | self.bold,
            italic: rhs.italic | self.italic,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Text(String, #[serde(skip)] TextStyle);

impl Text {
    pub fn new<T: ToString>(text: T) -> Self {
        Text(text.to_string(), Default::default())
    }

    pub fn style(self, style: TextStyle) -> Self {
        Text(self.0, style)
    }
}

impl ToBBCode for Text {
    fn to_bbcode(self) -> String {
        let mut text = self.0;

        if let Some(color) = self.1.color {
            text = format!("[color={color:}]{text:}[/color]");
        }

        if let Some(size) = self.1.size {
            text = format!("[size={size:}]{text:}[/size]");
        }

        if self.1.bold {
            text = format!("[b]{text:}[/b]");
        }

        if self.1.italic {
            text = format!("[i]{text:}[/i]");
        }

        return text;
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Number(f32, #[serde(skip)] TextStyle);

impl ToBBCode for Number {
    fn to_bbcode(self) -> String {
        Text(self.0.to_string(), self.1).to_bbcode()
    }
}

pub struct KeyValue(Text, Text);

impl KeyValue {
    pub fn new<T: ToString, U: ToString>(key: T, value: U) -> Self {
        Self(
            Text::new(key.to_string() + ":"),
            Text::new(" ".to_string() + &value.to_string()),
        )
    }

    pub fn style_key(self, key: TextStyle) -> Self {
        Self(Text(self.0 .0, key), self.1)
    }

    pub fn style_value(self, value: TextStyle) -> Self {
        Self(self.0, Text(self.1 .0, value))
    }

    pub fn style(self, key: TextStyle, value: TextStyle) -> Self {
        self.style_key(key).style_value(value)
    }
}

impl ToBBCode for KeyValue {
    fn to_bbcode(self) -> String {
        format!("{}{}", self.0.to_bbcode(), self.1.to_bbcode())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct List<T>(Vec<T>);

impl<T> FromIterator<T> for List<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        List(iter.into_iter().collect())
    }
}

impl<T: ToBBCode> ToBBCode for List<T> {
    fn to_bbcode(self) -> String {
        let mut out: String = Default::default();

        let mut iter = self.0.into_iter().map(ToBBCode::to_bbcode);
        let count = iter.len() - 1;
        for _ in 0..count {
            out += &iter.next().unwrap();
            out += "\n";
        }
        out += &iter.next().unwrap();

        out
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DoubleList<T>(Vec<T>);

impl<T> FromIterator<T> for DoubleList<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        DoubleList(iter.into_iter().collect())
    }
}

impl<T: ToBBCode> ToBBCode for DoubleList<T> {
    fn to_bbcode(self) -> String {
        let mut out: String = Default::default();

        let mut iter = self.0.into_iter().map(ToBBCode::to_bbcode);
        let count = iter.len() - 1;
        for _ in 0..count {
            out += &iter.next().unwrap();
            out += "\n";
            out += "\n";
        }
        out += &iter.next().unwrap();

        out
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Image(String);

impl Image {
    pub fn new<T: ToString>(url: T) -> Self {
        Image(url.to_string())
    }
}

impl ToBBCode for Image {
    fn to_bbcode(self) -> String {
        format!("[img]{}[/img]", self.0)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Spoiler<T>(T);

impl<T: ToBBCode> ToBBCode for Spoiler<T> {
    fn to_bbcode(self) -> String {
        format!("[spoiler]{}[/spoiler]", self.0.to_bbcode())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Enemy {
    name: String,
    image: String,
    health: usize,
    points: usize,
    description: Include<String>,
}

impl ToBBCode for Enemy {
    fn to_bbcode(self) -> String {
        format!(
            "{}\n\n{}\n\n{}\n\n{}",
            Text::new(self.name)
                .style(TextStyle::size(120) | TextStyle::bold() | TextStyle::italic())
                .to_bbcode(),
            Image::new(self.image).to_bbcode(),
            [
                KeyValue::new("Health", self.health).style_key(TextStyle::bold()),
                KeyValue::new("Points", self.points).style_key(TextStyle::bold())
            ]
            .into_iter()
            .collect::<List<_>>()
            .to_bbcode(),
            self.description.to_bbcode()
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Include<T>(String, #[serde(skip)] PhantomData<T>);

impl<T: for<'de> Deserialize<'de> + ToBBCode> ToBBCode for Include<T> {
    fn to_bbcode(self) -> String {
        println!("Loading {}", self.0);
        let s = std::fs::read_to_string(self.0).unwrap();
        let value: T = ron::from_str(&s).unwrap();
        value.to_bbcode()
    }
}

impl<T> Include<T> {
    pub fn new<U: ToString>(uri: U) -> Self {
        Self(uri.to_string(), Default::default())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Enemies(DoubleList<Include<List<Enemy>>>);

impl ToBBCode for Enemies {
    fn to_bbcode(self) -> String {
        self.0.to_bbcode()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TaromaruST {
    enemies: Include<Enemies>,
}

impl ToBBCode for TaromaruST {
    fn to_bbcode(self) -> String {
        self.enemies.to_bbcode()
    }
}

fn main() {
    let st = Include::<TaromaruST>::new("data/taromaru-st.ron").to_bbcode();
    println!("BBCode:\n{}", st);
}
