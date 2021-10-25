use crate::utils::{cross_fold, dedup_vec};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum JsonPathElement {
    Key(String),
    Iterator
}

pub type JsonPath = Vec<JsonPathElement>;

#[derive(Clone, PartialEq, Debug)]
pub enum JsonSchemaTree {
    PathNode(JsonPathElement, Vec<JsonSchemaTree>),
    PathEnd
}

pub type JsonSchema = Vec<JsonSchemaTree>;

impl JsonSchemaTree {
    pub fn has_same_root(&self, path: &JsonPath) -> bool {
        match (self, path.first()) {
            (JsonSchemaTree::PathNode(el, _), Some(h)) if *el == *h => true,
            _ => false
        }
    }

    pub fn add_path(self, path: &JsonPath) -> JsonSchemaTree {
        match (self, path.as_slice()) {
            (t, []) => t,
            (JsonSchemaTree::PathNode(el, ch), [h, tail @ ..]) if el == *h && ch.is_empty() =>
                JsonSchemaTree::PathNode(el, vec![to_schema_tree(Vec::from(tail))]),
            (JsonSchemaTree::PathNode(el, branches), [h, tail @ ..]) if el == *h =>
                JsonSchemaTree::PathNode(el, dedup_vec(append_path(branches, &Vec::from(tail)))),
            (JsonSchemaTree::PathEnd, p) => to_schema_tree(Vec::from(p)),
            (t, _) => t
        }
    }
}

fn to_schema_tree(path: JsonPath) -> JsonSchemaTree {
    match path.as_slice() {
        [] => JsonSchemaTree::PathEnd,
        [head, tail @ ..] =>
            JsonSchemaTree::PathNode(head.clone(), vec![to_schema_tree(tail.to_vec())])
    }
}

fn append_path(mut schema: JsonSchema, path: &JsonPath) -> JsonSchema {
    if schema.iter().any(|tr| tr.has_same_root(path)) {
        let mut out: JsonSchema = Vec::new();

        for tr in schema {
            out.push(tr.add_path(path));
        }

        out
    } else {
        schema.insert(0, to_schema_tree(path.clone()));
        schema
    }
}

pub fn to_schema(paths: Vec<JsonPath>) -> JsonSchema {
    paths.into_iter().fold(vec![], |schema, path| append_path(schema, &path))
}

pub enum JsonValueTree {
    ValueRoot(JsonPathElement, Vec<JsonValueTree>),
    SingleValue(JsonPathElement, Value),
    ValueArray(Vec<Value>),
    TreeArray(Vec<Vec<JsonValueTree>>) // Vec<JsonTree>
}

type JsonTree = Vec<JsonValueTree>;

fn extract_tree(value: Value, schema_tree: JsonSchemaTree) -> Option<JsonValueTree> {
    match schema_tree {
        JsonSchemaTree::PathEnd => None,
        JsonSchemaTree::PathNode(JsonPathElement::Key(k), children) =>
            match children.as_slice() {
                [JsonSchemaTree::PathEnd] =>
                    value.get(k.clone()).map(|v| JsonValueTree::SingleValue(JsonPathElement::Key(k.clone()), v.clone())),
                [nodes @ ..] => {
                    value.get(k.clone())
                        .map(|v| nodes.iter().flat_map(|ch| extract_tree(v.clone(), ch.clone())).collect::<Vec<_>>())
                        .map(|trees| JsonValueTree::ValueRoot(JsonPathElement::Key(k.clone()), trees))
                }
            },
        JsonSchemaTree::PathNode(JsonPathElement::Iterator, children) =>
            match children.as_slice() {
                [JsonSchemaTree::PathEnd] =>
                    value.as_array().map(move |values| JsonValueTree::ValueArray(values.clone())),
                [nodes @ ..] => {
                    value.as_array().map(|values| {
                        values.clone().iter()
                            .map(|v| nodes.iter().flat_map(|ch| extract_tree(v.clone(), ch.clone())).collect::<Vec<_>>())
                            .collect::<Vec<_>>()
                    }).map(|trees| JsonValueTree::TreeArray(trees))
                }
            }
    }
}

pub fn extract(schema: &JsonSchema, value: Value) -> JsonTree {
    schema.iter().flat_map(|tree| extract_tree(value.clone(), tree.clone())).collect()
}

fn gen_maps(flat: bool, jp: JsonPath, jvt: JsonValueTree) -> Vec<HashMap<String, Value>> {
    match jvt {
        JsonValueTree::ValueRoot(jpe, trees) => {
            let mut jp1 = jp.clone();
            jp1.push(jpe);
            cross_fold(trees.into_iter().map(|el| gen_maps(flat, jp1.clone(), el)).collect())
        },
        JsonValueTree::SingleValue(jpe, value) => {
            let mut jp1 = jp.clone();
            jp1.push(jpe);
            vec![HashMap::from([(json_path_string(jp1.clone()), value)])]
        },
        JsonValueTree::ValueArray(values) => {
            let mut jp1 = jp.clone();
            if !flat {
                jp1.push(JsonPathElement::Iterator);
            }
            values.into_iter().map(|v| HashMap::from([
                (json_path_string(jp1.clone()), v)
            ])).collect()
        },
        JsonValueTree::TreeArray(trees) => {
            let mut jp1 = jp.clone();
            if !flat {
                jp1.push(JsonPathElement::Iterator);
            }
            trees
                .into_iter()
                .flat_map(|jt|
                    cross_fold(
                        jt
                            .into_iter()
                            .map(|jvt| gen_maps(flat, jp1.clone(), jvt))
                            .collect::<Vec<_>>())
                )
                .collect::<Vec<_>>()
        }
    }
}

pub fn generate_tuples(flat: bool, tree: JsonTree) -> Vec<HashMap<String, Value>> {
    cross_fold(tree.into_iter().map(|jvt| gen_maps(flat, vec![], jvt)).collect())
}

pub fn json_path_string(path: JsonPath) -> String {
    path.iter().map(|pe| match pe {
        JsonPathElement::Key(k) => k,
        JsonPathElement::Iterator => "$"
    }).collect::<Vec<_>>().join(".")
}

#[cfg(test)]
mod schema_tests {
    use crate::schema::{JsonPathElement::*, JsonSchemaTree::*, append_path, to_schema_tree};

    #[test]
    fn has_same_root_single() {
        let sut = PathNode(Key("peka".to_string()), vec![]);

        assert!(sut.has_same_root(&vec![Key("peka".to_string())]));
        assert!(!sut.has_same_root(&vec![Key("yoba".to_string())]));
    }

    #[test]
    fn has_same_root_multi() {
        let sut = PathNode(Key("peka".to_string()), vec![
            PathNode(Key("yoba".to_string()), vec![]),
            PathNode(Key("pika".to_string()), vec![])
        ]);

        assert!(sut.has_same_root(&vec![Key("peka".to_string())]));
        assert!(sut.has_same_root(&vec![Key("peka".to_string()), Key("yoba".to_string())]));
        assert!(!sut.has_same_root(&vec![Key("yoba".to_string())]));
        assert!(!sut.has_same_root(&vec![Iterator]));
        assert!(sut.has_same_root(&vec![Key("peka".to_string()), Iterator]));
    }

    #[test]
    fn add_path() {
        let tree = PathNode(Key("peka".to_string()), vec![PathEnd]);
        let path = vec![Key("peka".to_string()), Key("yoba".to_string())];
        let sut = tree.add_path(&path);
        assert_eq!(sut,
                   PathNode(
                       Key("peka".to_string()),
                       vec![
                           PathNode(Key("yoba".to_string()), vec![PathEnd]),
                           PathEnd
                       ])
        );
    }

    #[test]
    fn append_path_test() {
        let schema = vec![PathNode(Key("peka".to_string()), vec![PathEnd])];
        let path = vec![Key("yoba".to_string())];
        let sut = append_path(schema, &path);
        assert_eq!(sut,
                   vec![
                       PathNode(Key("yoba".to_string()), vec![PathEnd]),
                       PathNode(Key("peka".to_string()), vec![PathEnd])
                   ]
        );
    }

    #[test]
    fn json_path_to_schema_tree() {
        let jp = vec![Key("peka".to_string()), Iterator, Key("yoba".to_string())];
        let sut = to_schema_tree(jp);
        assert_eq!(sut,
                   PathNode(
                       Key("peka".to_string()),
                       vec![PathNode(Iterator, vec![PathNode(Key("yoba".to_string()), vec![PathEnd])])]
                   )
        );
    }
}