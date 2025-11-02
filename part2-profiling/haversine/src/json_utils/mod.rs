use super::{
    Point, PointPair,
    json_parser::{
        ast::{Ast, KeyValuePair},
        parse_json,
    },
};

pub trait AstIterTools {
    fn as_object<'a>(&'a self) -> Option<&'a Vec<KeyValuePair>>;
    fn as_array<'a>(&'a self) -> Option<&'a Vec<Ast>>;
    fn as_f64<'a>(&'a self) -> Option<&'a f64>;
}

pub trait AstObjTools {
    fn find_by_key<'a>(&'a self, key: &str) -> Option<&'a Ast>;
}

impl AstObjTools for Vec<KeyValuePair> {
    fn find_by_key<'a>(&'a self, key: &str) -> Option<&'a Ast> {
        for ast in self {
            if ast.0 == key {
                return Some(&ast.1);
            }
        }
        return None;
    }
}

impl AstIterTools for Ast {
    fn as_array<'a>(&'a self) -> Option<&'a Vec<Ast>> {
        match self {
            Ast::Array(value) => Some(value),
            _ => None,
        }
    }
    fn as_object<'a>(&'a self) -> Option<&'a Vec<KeyValuePair>> {
        match self {
            Ast::Object(obj) => Some(obj),
            _ => None,
        }
    }
    fn as_f64<'a>(&'a self) -> Option<&'a f64> {
        match self {
            Ast::Number(value) => Some(value),
            _ => None,
        }
    }
}

pub fn prepare_data(json: String) -> JsonData {
    let result = parse_json(json).unwrap();

    let obj = result.as_object().expect("must be obj");
    let pairs = obj.find_by_key("pairs").unwrap();

    let point_ast_arr = pairs.as_array().unwrap();

    let mut pairs = Vec::with_capacity(point_ast_arr.len());

    for point_json in point_ast_arr {
        let obj = point_json.as_object().unwrap();

        let x0 = obj.find_by_key("x0").unwrap().as_f64().unwrap();
        let x1 = obj.find_by_key("x1").unwrap().as_f64().unwrap();
        let y0 = obj.find_by_key("y0").unwrap().as_f64().unwrap();
        let y1 = obj.find_by_key("y1").unwrap().as_f64().unwrap();

        pairs.push((Point { x: *x0, y: *y0 }, Point { x: *x1, y: *y1 }))
    }

    JsonData { pairs }
}

pub struct JsonData {
    pub pairs: Vec<PointPair>,
}
