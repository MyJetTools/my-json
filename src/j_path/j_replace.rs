use rust_extensions::StrOrString;

use crate::{
    json_reader::{JsonFirstLineIterator, JsonParseError},
    json_writer::JsonValueWriter,
};

pub fn j_update<'s, 'd>(
    json: &'s str,
    path: impl Into<StrOrString<'d>>,
    value_to_replace: impl JsonValueWriter,
) -> Result<String, JsonParseError> {
    let path: StrOrString = path.into();

    let mut result = String::new();

    j_replace_internal(
        json.as_bytes(),
        path.as_str(),
        &mut result,
        &value_to_replace,
    )?;

    Ok(result)
}

fn j_replace_internal<'s>(
    json: &'s [u8],
    path: &str,
    result: &mut String,
    value_to_replace: &impl JsonValueWriter,
) -> Result<(), JsonParseError> {
    result.push('{');
    if path.is_empty() {
        result.push('}');
        return Ok(());
    }
    let reader = JsonFirstLineIterator::new(json);

    let j_path_reader = crate::j_path::JPathReader::new(path);

    let j_prop_name = j_path_reader.get_prop_name();

    let mut index = 0;

    let mut value_injected = false;

    match j_prop_name {
        super::JPropName::Name(j_prop_name) => {
            while let Some(next) = reader.get_next() {
                let (key, value) = next.unwrap();

                let key = key.as_str()?;

                if index > 0 {
                    result.push(',');
                }

                index += 1;
                if key.as_str() == j_prop_name {
                    result.push('"');
                    result.push_str(key.as_str());
                    result.push('"');
                    result.push(':');

                    match j_path_reader.get_next_level_path() {
                        Some(next_level_path) => {
                            let data = &json[value.data.start..value.data.end];
                            j_replace_internal(data, next_level_path, result, value_to_replace)?;
                        }
                        None => {
                            value_injected = true;
                            value_to_replace.write(result);
                        }
                    }
                } else {
                    if let Some(value) = value.as_raw_str() {
                        result.push('"');
                        result.push_str(key.as_str());
                        result.push('"');
                        result.push(':');
                        result.push_str(value);
                    }
                }
            }
        }
        super::JPropName::ArrayAndIndex {
            j_prop_name: _,
            index: _,
        } => {

            //todo!("Array update is not implemented yet")
            /*
            while let Some(next) = reader.get_next() {
                let (key, value) = next.unwrap();

                let key = key.as_str()?;

                if key.as_str() == j_prop_name {
                    match j_path_reader.get_next_level_path() {
                        Some(next_level_path) => {
                            let data = &json[value.data.start..value.data.end];
                            return find_object_from_array(data, next_level_path, index);
                        }
                        None => {
                            let data = &json[value.data.start..value.data.end];
                            return find_object_from_array(data, "", index);
                        }
                    }
                }
            }
             */
        }
        super::JPropName::Array(_) => {
            return Err(JsonParseError::Other("Result is array".to_string()))
        }
    }

    if j_path_reader.get_next_level_path().is_none() {
        if !value_injected {
            if index > 0 {
                result.push(',');
            }

            result.push('"');
            result.push_str(j_path_reader.get_prop_name().as_str());
            result.push('"');
            result.push(':');
            value_to_replace.write(result);
        }
    }

    result.push('}');
    Ok(())
}

#[cfg(test)]
mod test {

    #[test]
    fn test_replace() {
        let json = r#"{
    "user_profile": {
        "name": null,
        "contact": {
            "email": null,
            "phone": null,
            "country_code_confirmed": null
        },
        "nationality": null,
        "time_zone": null
    },
    "preferences": {
        "locations": [],
        "budget": {
            "min": null,
            "max": null,
            "currency": null
        },
        "property_type": null,
        "bhk": null,
        "completion_year": null,
        "brand_preference": null,
        "special_features": []
    },
    "conversation_state": {
        "stage": "pitching",
        "last_tool_called": null,
        "last_tool_parameters": {},
        "last_pitched_property": {
            "id": null,
            "name": "The Astera, Interiors by Aston Martin"
        },
        "pending_questions": []
    },
    "constraints": [],
    "objections": [],
    "follow_up_commitments": []
}"#;

        let result = super::j_update(json, "user_profile.name", "Ivan").unwrap();

        println!("{}", result);

        println!("------");

        let result = super::j_update(
            result.as_str(),
            "user_profile.contact.email",
            "email@email.com",
        )
        .unwrap();

        println!("{}", result);
    }

    #[test]
    fn test_insert_to_not_existing_value() {
        let json = r#"{
    "user_profile": {"contact":{"email": null,"phone": null,"country_code_confirmed": null},"nationality": null,"time_zone": null}}"#;

        let result = super::j_update(json, "user_profile.name", "Ivan").unwrap();

        println!("{}", result);

        let value = crate::j_path::get_value(result.as_bytes(), "user_profile.name")
            .unwrap()
            .unwrap();

        assert_eq!("Ivan", value.as_str().unwrap().as_str());

        println!("------");

        let result = super::j_update(result.as_str(), "other_key", "OtherValue").unwrap();

        let value = crate::j_path::get_value(result.as_bytes(), "other_key")
            .unwrap()
            .unwrap();

        assert_eq!("OtherValue", value.as_str().unwrap().as_str());

        println!("{}", result);

        println!("------");
    }
}
