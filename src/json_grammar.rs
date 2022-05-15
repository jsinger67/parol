use crate::json_grammar_trait::*;
use miette::Result;
use std::fmt::{Debug, Display, Error, Formatter};

impl Display for Json<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}", self.value)
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Value::Value0(v) => write!(f, "{}", v.string.string.symbol),
            Value::Value1(v) => write!(f, "{}", v.number.number.symbol),
            Value::Value2(v) => write!(f, "{{{}}}", v.object.object_suffix),
            Value::Value3(v) => write!(f, "[{}]", v.array.array_suffix),
            Value::Value4(_) => write!(f, "true"),
            Value::Value5(_) => write!(f, "false"),
            Value::Value6(_) => write!(f, "null"),
        }
    }
}

impl Display for ObjectSuffix<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            ObjectSuffix::ObjectSuffix0(o) => write!(
                f,
                "{}{}",
                o.pair,
                o.object_list
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<std::string::String>>()
                    .join("")
            ),
            ObjectSuffix::ObjectSuffix1(_) => Ok(()),
        }
    }
}

impl Display for ObjectList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{} {}", self.comma.symbol, self.pair)
    }
}

impl Display for ArraySuffix<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            ArraySuffix::ArraySuffix0(a) => write!(
                f,
                "{}{}",
                a.value,
                a.array_list
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<std::string::String>>()
                    .join("")
            ),
            ArraySuffix::ArraySuffix1(_) => Ok(()),
        }
    }
}

impl Display for ArrayList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{} {}", self.comma.symbol, self.value)
    }
}

impl Display for Pair<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}: {}", self.string.string.symbol, self.value)
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
