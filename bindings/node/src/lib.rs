// Node.js bindings for WalDB using Neon
// Provides a JavaScript API that mirrors the Firebase RTDB interface

use neon::prelude::*;
use std::sync::{Arc, Mutex};
use std::path::Path;
use waldb::Store;

// Wrapper to make Store Send + Sync for Neon
struct StoreWrapper {
    store: Arc<Mutex<Store>>,
}

impl Finalize for StoreWrapper {}

type BoxedStore = JsBox<StoreWrapper>;

// Open or create a WalDB store
fn open_store(mut cx: FunctionContext) -> JsResult<BoxedStore> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    
    match Store::open(Path::new(&path)) {
        Ok(store) => {
            let wrapper = StoreWrapper {
                store: Arc::new(Mutex::new(store)),
            };
            Ok(cx.boxed(wrapper))
        }
        Err(e) => cx.throw_error(format!("Failed to open store: {}", e))
    }
}

// Set a value at a path
fn set(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let store = cx.argument::<BoxedStore>(0)?;
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    let value = cx.argument::<JsString>(2)?.value(&mut cx);
    let force = cx.argument_opt(3)
        .map(|arg| arg.downcast_or_throw::<JsBoolean, _>(&mut cx).map(|b| b.value(&mut cx)))
        .transpose()?
        .unwrap_or(false);
    
    match store.store.lock().unwrap().set(&key, &value, force) {
        Ok(_) => Ok(cx.undefined()),
        Err(e) => cx.throw_error(format!("Failed to set value: {}", e))
    }
}

// Get a value or subtree at a path
fn get(mut cx: FunctionContext) -> JsResult<JsValue> {
    let store = cx.argument::<BoxedStore>(0)?;
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    
    match store.store.lock().unwrap().get(&key) {
        Ok(Some(value)) => {
            if value.starts_with('{') || value.starts_with('[') {
                // Try to parse as JSON
                match serde_json::from_str::<serde_json::Value>(&value) {
                    Ok(json_val) => {
                        // Convert serde_json::Value to JsValue
                        json_to_js_value(&mut cx, &json_val)
                    }
                    Err(_) => Ok(cx.string(value).as_value(&mut cx))
                }
            } else {
                Ok(cx.string(value).as_value(&mut cx))
            }
        }
        Ok(None) => Ok(cx.null().as_value(&mut cx)),
        Err(e) => cx.throw_error(format!("Failed to get value: {}", e))
    }
}

// Delete a key and its subtree
fn delete(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let store = cx.argument::<BoxedStore>(0)?;
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    
    match store.store.lock().unwrap().delete(&key) {
        Ok(_) => Ok(cx.undefined()),
        Err(e) => cx.throw_error(format!("Failed to delete: {}", e))
    }
}

// Check if a key exists
fn exists(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let store = cx.argument::<BoxedStore>(0)?;
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    
    match store.store.lock().unwrap().exists(&key) {
        Ok(exists) => Ok(cx.boolean(exists)),
        Err(e) => cx.throw_error(format!("Failed to check existence: {}", e))
    }
}

// Get values matching a pattern
fn get_pattern(mut cx: FunctionContext) -> JsResult<JsObject> {
    let store = cx.argument::<BoxedStore>(0)?;
    let pattern = cx.argument::<JsString>(1)?.value(&mut cx);
    
    match store.store.lock().unwrap().get_pattern(&pattern) {
        Ok(results) => {
            let obj = cx.empty_object();
            for (key, value) in results {
                let js_key = cx.string(key);
                let js_value = cx.string(value);
                obj.set(&mut cx, js_key, js_value)?;
            }
            Ok(obj)
        }
        Err(e) => cx.throw_error(format!("Failed to get pattern: {}", e))
    }
}

// Get values in a range
fn get_range(mut cx: FunctionContext) -> JsResult<JsObject> {
    let store = cx.argument::<BoxedStore>(0)?;
    let start = cx.argument::<JsString>(1)?.value(&mut cx);
    let end = cx.argument::<JsString>(2)?.value(&mut cx);
    
    match store.store.lock().unwrap().get_range(&start, &end) {
        Ok(results) => {
            let obj = cx.empty_object();
            for (key, value) in results {
                let js_key = cx.string(key);
                let js_value = cx.string(value);
                obj.set(&mut cx, js_key, js_value)?;
            }
            Ok(obj)
        }
        Err(e) => cx.throw_error(format!("Failed to get range: {}", e))
    }
}

// List all keys with a prefix
fn list_keys(mut cx: FunctionContext) -> JsResult<JsArray> {
    let store = cx.argument::<BoxedStore>(0)?;
    let prefix = cx.argument::<JsString>(1)?.value(&mut cx);
    
    match store.store.lock().unwrap().list_keys(&prefix) {
        Ok(keys) => {
            let array = cx.empty_array();
            for (i, key) in keys.into_iter().enumerate() {
                let js_key = cx.string(key);
                array.set(&mut cx, i as u32, js_key)?;
            }
            Ok(array)
        }
        Err(e) => cx.throw_error(format!("Failed to list keys: {}", e))
    }
}

// Flush the WAL to disk
fn flush(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let store = cx.argument::<BoxedStore>(0)?;
    
    match store.store.lock().unwrap().flush() {
        Ok(_) => Ok(cx.undefined()),
        Err(e) => cx.throw_error(format!("Failed to flush: {}", e))
    }
}

// Helper function to convert serde_json::Value to JsValue
fn json_to_js_value(cx: &mut FunctionContext, value: &serde_json::Value) -> JsResult<JsValue> {
    match value {
        serde_json::Value::Null => Ok(cx.null().as_value(cx)),
        serde_json::Value::Bool(b) => Ok(cx.boolean(*b).as_value(cx)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(cx.number(i as f64).as_value(cx))
            } else if let Some(f) = n.as_f64() {
                Ok(cx.number(f).as_value(cx))
            } else {
                Ok(cx.null().as_value(cx))
            }
        }
        serde_json::Value::String(s) => Ok(cx.string(s).as_value(cx)),
        serde_json::Value::Array(arr) => {
            let js_array = cx.empty_array();
            for (i, item) in arr.iter().enumerate() {
                let js_item = json_to_js_value(cx, item)?;
                js_array.set(cx, i as u32, js_item)?;
            }
            Ok(js_array.as_value(cx))
        }
        serde_json::Value::Object(map) => {
            let js_obj = cx.empty_object();
            for (key, val) in map {
                let js_key = cx.string(key);
                let js_val = json_to_js_value(cx, val)?;
                js_obj.set(cx, js_key, js_val)?;
            }
            Ok(js_obj.as_value(cx))
        }
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("open", open_store)?;
    cx.export_function("set", set)?;
    cx.export_function("get", get)?;
    cx.export_function("delete", delete)?;
    cx.export_function("exists", exists)?;
    cx.export_function("getPattern", get_pattern)?;
    cx.export_function("getRange", get_range)?;
    cx.export_function("listKeys", list_keys)?;
    cx.export_function("flush", flush)?;
    Ok(())
}