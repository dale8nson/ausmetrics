use std::{
    cell::{Cell, RefCell},
    collections::{BTreeSet, HashSet},
    ops::Deref,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
    vec::IntoIter,
};

use futures::{StreamExt, stream};
use log::{debug, info};

use async_graphql::{
    Value,
    dynamic::{
        Field, FieldFuture, FieldValue, Object, Scalar, Schema, SchemaBuilder, TypeRef,
        indexmap::IndexMap,
    },
    *,
};
use reqwest::Client;

// static FIELDS: HashSet<&str> = HashSet::<&str>::new();

const STRUCTURE_TYPES: [&'static str; 8] = [
    "datastructure",
    "dataflow",
    "codelist",
    "conceptscheme",
    "categoryscheme",
    "contentconstraint",
    "actualconstraint",
    // "agencyscheme",
    "categorisation",
    // "hierarchicalcodelist",
];

fn to_typeref(name: &str, value: Value) -> TypeRef {
    // if name == "dataStructures" {
    //     debug!("{name:?}: {value:?}");
    // }
    match value {
        Value::Object(_) => TypeRef::named(name),
        Value::List(list) => {
            if let Some(first) = list.get(0) {
                // if the list contains objects, build a list of that object type;
                // otherwise build a list of the scalar type (e.g. Int, Float, String)
                match first {
                    Value::Object(_) => TypeRef::named_list(name),
                    _ => {
                        let inner = to_typeref(name, first.clone());
                        info!("{:?}", inner);
                        TypeRef::List(Box::new(inner))
                    }
                }
            } else {
                // empty list ⇒ default to list of strings
                TypeRef::named_list(TypeRef::STRING)
            }
        }
        Value::String(s) if s.to_lowercase() == "id" => TypeRef::named(TypeRef::ID),
        Value::String(_) => TypeRef::named(TypeRef::STRING),
        Value::Number(num) if num.is_f64() => TypeRef::named(TypeRef::FLOAT),
        Value::Number(_) => TypeRef::named(TypeRef::INT),
        Value::Boolean(_) => TypeRef::named(TypeRef::BOOLEAN),
        Value::Binary(_) => TypeRef::named(TypeRef::UPLOAD),
        Value::Enum(_) => TypeRef::named(TypeRef::STRING),
        _ => TypeRef::named(TypeRef::STRING),
    }
}

fn to_field(name: Name, value: Value, ty_override: Option<TypeRef>) -> Field {
    let field_name = name.to_string();
    // Use the override type if provided; otherwise, infer it from the JSON value.
    let ty = ty_override.unwrap_or_else(|| to_typeref(&field_name, value.clone()));

    // Remember if this field is a list type so we can return an empty list
    // instead of `Null` when the JSON key is missing.
    let is_list = matches!(ty, TypeRef::List(_));

    Field::new(&field_name.clone(), ty, move |ctx| {
        let field_name = field_name.clone();
        let is_list = is_list;
        FieldFuture::new(async move {
            let parent_value = ctx.parent_value.as_value();

            let child_value = if let Some(Value::Object(map)) = parent_value {
                map.get(&Name::new(&field_name))
                    .cloned()
                    .unwrap_or_else(|| {
                        if is_list {
                            Value::List(vec![])
                        } else {
                            Value::Null
                        }
                    })
            } else if is_list {
                Value::List(vec![])
            } else {
                Value::Null
            };

            Ok(Some(child_value))
        })
    })
}

fn to_object(type_name: Name, map: IndexMap<Name, Value>) -> (Object, Vec<Object>) {
    debug!("{:?}", type_name.as_str());
    let mut objs = Vec::<Object>::new();

    if map.is_empty() {
        return (Object::new(type_name.as_str()), objs);
    }

    let type_name_str = type_name.as_str().to_string();
    let mut obj = Object::new(&type_name_str);
    let mut fields = HashSet::<&str>::new();

    for (field_name, value) in map.iter() {
        let field_key = field_name.as_str();

        debug!(
            "{:?}: {:?}",
            field_key,
            to_typeref(field_key, value.clone())
        );

        match value.clone() {
            Value::Object(index_map) => {
                if !index_map.is_empty() {
                    // Construct a unique type name for this nested object based on the parent type.
                    let nested_type_name = format!("{}_{}", type_name_str, field_key);
                    let (nested_obj, nested_objs) =
                        to_object(Name::new(&nested_type_name), index_map);

                    objs.extend(nested_objs);

                    if fields.insert(field_key) {
                        let field_type = TypeRef::named(&nested_type_name);
                        obj = obj.field(to_field(
                            field_name.clone(),
                            value.clone(),
                            Some(field_type),
                        ));
                    }

                    objs.push(nested_obj);
                }
            }

            Value::List(lst) => {
                // Handle empty lists first: we can't infer element structure.
                if lst.is_empty() {
                    if fields.insert(field_key) {
                        let list_type = TypeRef::named_list(TypeRef::STRING);
                        obj = obj.field(to_field(
                            field_name.clone(),
                            Value::List(vec![]),
                            Some(list_type),
                        ));
                    }
                    continue;
                }

                match lst[0].clone() {
                    Value::Object(index_map) => {
                        if !index_map.is_empty() {
                            // Unique type name for list element objects as well.
                            let nested_type_name = format!("{}_{}", type_name_str, field_key);
                            let (nested_obj, nested_objs) =
                                to_object(Name::new(&nested_type_name), index_map);

                            objs.extend(nested_objs);

                            if fields.insert(field_key) {
                                let field_type = TypeRef::named_list(&nested_type_name);
                                obj = obj.field(to_field(
                                    field_name.clone(),
                                    value.clone(),
                                    Some(field_type),
                                ));
                            }

                            objs.push(nested_obj);
                        }
                    }
                    _ => {
                        // List of scalars: let to_typeref infer the scalar type.
                        if fields.insert(field_key) {
                            obj = obj.field(to_field(field_name.clone(), value.clone(), None));
                        }
                    }
                }
            }

            _ => {
                debug!(
                    "field: {:?}: {:?}",
                    field_name.as_str(),
                    to_typeref(field_name.as_str(), value.clone())
                );
                if fields.insert(field_key) {
                    obj = obj.field(to_field(field_name.clone(), value.clone(), None));
                }
            }
        }
    }

    debug!("obj: {:?}", obj.type_name());
    (obj, objs)
}

pub async fn get_schema() -> Schema {
    let path = PathBuf::from("https://data.api.abs.gov.au/rest");
    let client = Arc::new(reqwest::Client::new());

    let mut query = Object::new("Query");
    let mut schema = Schema::build(query.type_name(), None, None);
    let mut meta_map = IndexMap::<Name, Value>::new();
    // Custom root meta type that aggregates ABS structure types.
    // Use a unique name so it does not collide with any "meta" objects
    // that may appear inside the ABS JSON and be turned into GraphQL
    // types by `to_object`.
    let mut root_meta = Object::new("AbsStructuresMeta");

    for ty in STRUCTURE_TYPES.iter() {
        debug!("ty: {ty}");
        let type_name = *ty;
        let mut url = path.clone();

        url.push(format!("{ty}/ABS?detail=allstubs"));
        debug!("url: {url:?}");

        let res = client
            .get(url.to_str().unwrap())
            .header("accept", "application/vnd.sdmx.structure+json")
            .header("user-agent", "ausmetrics/0.1.0")
            .send()
            .await
            .expect("Error retrieving data");

        let value = res.json::<Value>().await.expect("Unable to parse JSON");

        meta_map.insert(Name::new(type_name), value.clone());

        if let Value::Object(ref index_map) = value.clone() {
            let (ob, obs) = to_object(Name::new(type_name), index_map.clone());
            schema = schema.register(ob);
            for o in obs {
                schema = schema.register(o);
            }

            // add this structure as a field on the root meta object
            let v = value.clone();
            root_meta = root_meta.field(to_field(Name::new(type_name), v, None));
        }
    }

    let meta_data = meta_map.clone();
    // Expose the aggregated structures under Query.meta, whose type is AbsStructuresMeta
    query = query.field(Field::new(
        "meta",
        TypeRef::named("AbsStructuresMeta"),
        move |_| FieldFuture::from_value(Some(Value::Object(meta_data.clone()))),
    ));

    schema = schema
        .data(client.clone())
        .register(query)
        .register(root_meta);
    let schema = schema.finish().expect("unable to parse schema");
    let dataflow_ids = schema
        .execute("{meta {dataflow { data { dataflows { id }}}}}")
        .await
        .data;

    schema
}
