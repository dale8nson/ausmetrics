use crate::error::GQLError;
use actix_web::Either;
use async_graphql::{
    EmptyMutation, EmptySubscription, Json, Name, Number, OutputType, Value,
    dynamic::{
        Field, FieldFuture, FieldValue, Interface, Object, Schema, TypeRef, indexmap::IndexMap,
    },
    to_value,
};
use async_graphql_value::ConstValue;
use core::error::Error;
use serde_json::from_value;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{HashMap, hash_map::Keys},
    ops::{ControlFlow, Deref},
    path::PathBuf,
    rc::Rc,
    str::FromStr,
    sync::{Mutex, RwLock},
};
use yaml_rust2::{
    Yaml,
    yaml::{Hash, LoadError, YamlDecoder},
};

fn yaml_error_cb(
    malformation_length: u8,
    bytes_read_after_malformation: u8,
    input_at_malformation: &[u8],
    output: &mut String,
) -> ControlFlow<Cow<'static, str>> {
    let input = str::from_utf8(input_at_malformation);
    println!("input: {input:?}");
    println!("{output}");

    ControlFlow::Break(Cow::from("YAML broken :("))
}

pub fn parse_yaml_doc(path: PathBuf) -> Result<Vec<Yaml>, GQLError> {
    let src = std::fs::File::open(path)?;
    let buf_reader = std::io::BufReader::new(src);

    let yaml = YamlDecoder::read(buf_reader)
        .encoding_trap(yaml_rust2::yaml::YAMLDecodingTrap::Call(yaml_error_cb))
        .decode()
        .map_err(|load_error| match load_error {
            LoadError::IO(err) => GQLError::Io(err),
            LoadError::Decode(err) => GQLError::Io(std::io::Error::other(err)),
            LoadError::Scan(err) => GQLError::Io(std::io::Error::other(err)),
        })?;

    Ok(yaml)
}

fn getln() {
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

pub fn yaml_to_value(yaml: Yaml) -> Value {
    // println!("{yaml:#?}");
    // getln();
    match yaml {
        Yaml::String(string) => to_value(string).unwrap(),
        Yaml::Integer(int) => Value::Number(Number::from_f64(int as f64).unwrap()),
        Yaml::Real(real) => Value::Number(Number::from_str(real.as_str()).unwrap()),
        Yaml::Boolean(b) => Value::Boolean(b),
        Yaml::Array(arr) => Value::List(
            arr.into_iter()
                .map(|yaml| yaml_to_value(yaml))
                .collect::<Vec<ConstValue>>(),
        ),
        Yaml::Hash(hash) => Value::Object(IndexMap::<Name, ConstValue>::from_iter(
            hash.into_iter().map(|(k, v)| {
                (
                    Name::new(if let Yaml::String(name) = k.clone() {
                        name
                    } else {
                        panic!("unexpected key type")
                    }),
                    yaml_to_value(v),
                )
            }), // .collect::<Vec<(Name, ConstValue)>>()
        )),
        Yaml::Alias(_) => Value::Null,
        Yaml::BadValue => Value::Null,
        Yaml::Null => Value::Null,
    }
}

pub fn parse_yaml<'a>(yaml: Yaml) -> FieldValue<'a> {
    // println!("yaml before: {yaml:#?}");
    // std::io::stdin().read_line(&mut String::new())?;
    println!("{yaml:#?}");
    getln();

    match yaml.clone() {
        Yaml::Integer(int) => {
            FieldValue::value(to_value(int).unwrap()).with_type(Cow::Borrowed(TypeRef::INT))
        }
        Yaml::Real(real) => FieldValue::value(f64::from_str(real.as_str()).unwrap())
            .with_type(Cow::Borrowed(TypeRef::FLOAT)),
        Yaml::String(string) => {
            FieldValue::value(to_value(string).unwrap()).with_type(Cow::Borrowed(TypeRef::STRING))
        }

        Yaml::Array(arr) => FieldValue::list(arr.into_iter().map(|yaml| parse_yaml(yaml))),
        Yaml::Hash(_) => FieldValue::value(yaml_to_value(yaml)),

        Yaml::Boolean(b) => {
            FieldValue::value(Value::Boolean(b)).with_type(Cow::Borrowed(TypeRef::BOOLEAN))
        }

        Yaml::Alias(_) | Yaml::BadValue | Yaml::Null => FieldValue::NULL,
    }
    // println!("value: {:#?}", value);
    // getln();
    // println!("yaml after: {yaml:#?}");
    // std::io::stdin().read_line(&mut String::new()).unwrap();
}

pub fn to_typeref(k: String, v: Yaml) -> TypeRef {
    match v {
        Yaml::Integer(_) => TypeRef::named(TypeRef::INT),
        Yaml::Real(_) => TypeRef::named(TypeRef::FLOAT),
        Yaml::String(_) => TypeRef::named(TypeRef::STRING),
        Yaml::Boolean(_) => TypeRef::named(TypeRef::BOOLEAN),
        Yaml::Array(arr) => TypeRef::named(to_typeref(k, arr[0].clone()).type_name()),
        _ => TypeRef::named(k),
    }
}

pub fn to_gql(yaml_doc: Vec<Yaml>) -> Result<Schema, GQLError> {
    // In async-graphql's *dynamic* API, `Schema::build(...)` expects the **root type names**
    // (e.g. "Query"), and you then `.register(...)` the actual `Object`s/`Enum`s/etc.
    // A `Value` is *runtime data*, not a schema definition, so `Value::to_string()` will
    // never be valid SDL here.

    // let mut query = Object::new("Query");

    // Keep your YAML->Value conversion, but expose it as a field so we can confirm the
    // parser works while we work on real type/field generation.
    // This produces a usable schema instead of attempting to build from a Value string.
    let yaml = yaml_doc.get(0).unwrap().clone();
    let hash = yaml.into_hash().unwrap();
    let paths = hash
        .get_key_value(&Yaml::from_str("paths"))
        .unwrap()
        .1
        .clone()
        .into_hash()
        .unwrap()
        .keys()
        .map(|k| k.clone().into_string().unwrap())
        .collect::<Vec<String>>();

    println!("{paths:#?}");

    let components = hash
        .get_key_value(&Yaml::from_str("components"))
        .unwrap()
        .1
        .clone()
        .into_hash()
        .unwrap();

    let params = components
        .get_key_value(&Yaml::from_str("parameters"))
        .unwrap()
        .1
        .clone()
        .into_hash()
        .unwrap();

    let parameter_names = params
        .clone()
        .keys()
        .map(|y| {
            format!(
                "{}",
                y.clone().into_string().unwrap(),
                // v.clone().into_string().unwrap()
            )
        })
        .collect::<Vec<String>>();

    println!("{parameter_names:#?}");

    // let mut parameters = Object::new("parameters");

    let (object, types) = parse_hash(&params, parameter_names, Object::new("parameters"));
    // println!("parameters.len(): {}", parameters.len());
    let query = object;
    println!("{query:#?}");
    let type_name = query.type_name();

    let mut schema_builder = Schema::build(type_name, None, None);
    // let sb = schema_builder.get_mut();
    schema_builder = [query]
        .into_iter()
        .chain(types)
        .fold(schema_builder, |acc, ty| acc.register(ty));
    let schema = schema_builder.finish().unwrap();
    // let yaml_value = yaml_doc
    //     .get(0)
    //     .cloned()
    //     .map(|doc| parse_yaml(doc))
    //     .unwrap_or(FieldValue::NULL);

    // let query = Field::new("Query", )
    // A simple field that returns the parsed YAML as a String.
    // (If you want a real JSON scalar later, we can register a `Scalar` named "JSON".)
    //
    // let ty = query.write().unwrap().type_name().to_owned();
    // query = query.write().unwrap().into.field(Field::new("query", TypeRef::named_nn(ty), move |_ctx| {
    //     let yaml_value = yaml_value.clone();
    //     FieldFuture::new(async move { Ok(Some(yaml_value)) })
    // }));

    // Build the dynamic schema properly: build with the root type name(s), then register.
    // let schema = Schema::build(query.type_name(), None, None)
    //     .register(query)
    //     .finish()
    //     .map_err(|e| {
    //         // SchemaError is just a String wrapper; make sure it doesn't get lost.
    //         // This keeps the error message readable in logs.
    //         GQLError::Io(std::io::Error::other(e.0))
    //     })?;

    println!("{}", schema.sdl());

    Ok(schema)
}

fn into_field(k: Yaml, v: Yaml) -> Field {
    let mut name = k.clone().into_string().unwrap();
    name = name.replace("-", "_");
    // if name == "schema" {
    //     name = String::from("schema_");
    // }
    let type_ref = to_typeref(name.clone(), v.clone());
    Field::new(name, type_ref, move |_| {
        FieldFuture::from_value(Some(yaml_to_value(v.clone())))
    })
}

fn parse_hash<'a>(hash: &'a Hash, keys: Vec<String>, mut object: Object) -> (Object, Vec<Object>) {
    println!("Object: {}", object.type_name());
    let mut types = Vec::<Object>::new();
    for mut name in keys {
        name = name.replace("_", "-");
        // if name == "schema_" {
        //     name = String::from("schema");
        // }
        let (k, v) = hash.get_key_value(&Yaml::from_str(&name)).unwrap();
        let k = k.clone();

        let mut type_name = k.clone().into_string().unwrap();
        type_name = type_name.replace("-", "_");
        // if type_name == "schema" {
        //     type_name = String::from("schema_")
        // }

        let v = v.clone();
        if let Yaml::Hash(hash) = v.clone() {
            let keys = hash
                .keys()
                .cloned()
                .map(|key| {
                    let mut key = key.into_string().unwrap();
                    key = key.replace("-", "_");
                    // if key.as_str() == "schema" {
                    //     String::from("schema_")
                    // } else {
                    key
                    // }
                })
                .collect::<Vec<String>>()
                .clone();

            let (root, tys) = parse_hash(&hash, keys, Object::new(type_name));
            types.extend([root].into_iter().chain(tys));
        }

        let field = into_field(k.clone(), v.clone());
        object = object.field(field);
    }

    (object, types)
}
