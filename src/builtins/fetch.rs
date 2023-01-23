use crate::JsonValue;
use anyhow::Context;
use serde_json::Value;

#[js::bind(object, public)]
#[allow(unused_variables)]
pub(crate) async fn fetch(args: JsonValue) -> Result<JsonValue, js::Error> {
    let ret = do_fetch(args.0)
        .await
        .map_err(|e| js::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(JsonValue(ret))
}

#[inline(always)]
async fn do_fetch(args: Value) -> anyhow::Result<Value> {
    // use reqwest to fetch the url and return the result
    // https://docs.rs/reqwest/0.11.4/reqwest/
    let client = reqwest::Client::new();
    match args {
        Value::String(url) => {
            let res = client.get(url).send().await?;
            let data: Value = res.json().await?;
            Ok(data)
        }
        Value::Object(obj) => {
            let url = obj
                .get("url")
                .and_then(|v| v.as_str())
                .context("args should include url")?;
            let method = obj.get("method").and_then(|v| v.as_str()).unwrap_or("get");
            let body = obj.get("body");
            let builder = client.request(method.parse().unwrap_or_default(), url);

            let builder = match body {
                Some(Value::String(body)) => builder.body(body.to_string()),
                Some(Value::Object(body)) => builder.json(body),
                _ => builder,
            };
            let res = builder.send().await?;
            let data: Value = res.json().await?;
            Ok(data)
        }
        _ => unimplemented!("Not supported value type"),
    }
}
