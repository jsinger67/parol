use crate::json_grammar_trait::*;
use anyhow::Result;
use std::fmt::{Debug, Display, Error, Formatter};

impl Display for Json<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}", self.value)
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Value::String(v) => write!(f, "{}", v.string.string.text()),
            Value::Number(v) => write!(f, "{}", v.number.number.text()),
            Value::Object(v) => write!(f, "{{{}}}", v.object.object_suffix),
            Value::Array(v) => write!(f, "[{}]", v.array.array_suffix),
            Value::True(_) => write!(f, "true"),
            Value::False(_) => write!(f, "false"),
            Value::Null(_) => write!(f, "null"),
        }
    }
}

impl Display for ObjectSuffix<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            ObjectSuffix::PairObjectListRBrace(o) => write!(
                f,
                "{}{}",
                o.pair,
                o.object_list
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<std::string::String>>()
                    .join("")
            ),
            ObjectSuffix::RBrace(_) => Ok(()),
        }
    }
}

impl Display for ObjectList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, ", {}", self.pair)
    }
}

impl Display for ArraySuffix<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            ArraySuffix::ValueArrayListRBracket(a) => write!(
                f,
                "{}{}",
                a.value,
                a.array_list
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<std::string::String>>()
                    .join("")
            ),
            ArraySuffix::RBracket(_) => Ok(()),
        }
    }
}

impl Display for ArrayList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, ", {}", self.value)
    }
}

impl Display for Pair<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}: {}", self.string.string.text(), self.value)
    }
}

///
/// Data structure used to build up a json structure during parsing
///
#[derive(Debug, Default)]
pub struct JsonGrammar<'t> {
    pub json: Option<Json<'t>>,
}

impl Display for JsonGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.json {
            Some(json) => write!(f, "{}", json),
            None => write!(f, "No parse result"),
        }
    }
}

impl JsonGrammar<'_> {
    pub fn new() -> Self {
        JsonGrammar::default()
    }
}

impl<'t> JsonGrammarTrait<'t> for JsonGrammar<'t> {
    fn json(&mut self, arg: &Json<'t>) -> Result<()> {
        self.json = Some(arg.clone());
        Ok(())
    }
}
