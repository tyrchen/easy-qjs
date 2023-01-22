use crate::JsonValue;
use js::Rest;

#[js::bind(object, public)]
#[allow(unused_variables)]
pub(crate) async fn fetch(args: Rest<JsonValue>) -> Result<JsonValue, js::Error> {
    Ok(JsonValue::default())
}
