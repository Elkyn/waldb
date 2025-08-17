use neon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

// Include the WalDB implementation directly 
mod waldb_store {
    include!("../../../waldb.rs");
}

use waldb_store::Store;

// Global store cache - stores are thread-safe and can be shared
static STORE_CACHE: Lazy<Arc<Mutex<HashMap<String, Arc<Store>>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// Helper function to get or create a cached store
fn get_or_create_store(path: &str) -> Result<Arc<Store>, String> {
    let mut cache = STORE_CACHE.lock().map_err(|e| format!("Cache lock error: {}", e))?;
    
    if let Some(store) = cache.get(path) {
        Ok(Arc::clone(store))
    } else {
        match Store::open(Path::new(path)) {
            Ok(store) => {
                let store_arc = Arc::new(store);
                cache.insert(path.to_string(), Arc::clone(&store_arc));
                Ok(store_arc)
            }
            Err(e) => Err(format!("Failed to open store: {}", e))
        }
    }
}

fn open_store(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    
    match get_or_create_store(&path) {
        Ok(_) => Ok(cx.string(path)),
        Err(e) => cx.throw_error(e)
    }
}

fn set_value(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    let value = cx.argument::<JsString>(2)?.value(&mut cx);
    let force = cx.argument_opt(3)
        .and_then(|arg| arg.downcast::<JsBoolean, _>(&mut cx).ok())
        .map(|b| b.value(&mut cx))
        .unwrap_or(false);
    
    match get_or_create_store(&path) {
        Ok(store) => {
            match store.set(&key, &value, force) {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("Failed to set value: {}", e))
            }
        }
        Err(e) => cx.throw_error(e)
    }
}

fn get_value(mut cx: FunctionContext) -> JsResult<JsValue> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    
    match get_or_create_store(&path) {
        Ok(store) => {
            match store.get(&key) {
                Ok(Some(value)) => {
                    if value.starts_with('{') || value.starts_with('[') {
                        // Try to parse as JSON
                        match serde_json::from_str::<serde_json::Value>(&value) {
                            Ok(json_val) => json_to_js(&mut cx, &json_val),
                            Err(_) => Ok(cx.string(value).upcast())
                        }
                    } else {
                        Ok(cx.string(value).upcast())
                    }
                }
                Ok(None) => Ok(cx.null().upcast()),
                Err(e) => cx.throw_error(format!("Failed to get value: {}", e))
            }
        }
        Err(e) => cx.throw_error(e)
    }
}

fn delete_key(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    
    match get_or_create_store(&path) {
        Ok(store) => {
            match store.delete(&key) {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("Failed to delete: {}", e))
            }
        }
        Err(e) => cx.throw_error(e)
    }
}

fn get_pattern_matches(mut cx: FunctionContext) -> JsResult<JsObject> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    let pattern = cx.argument::<JsString>(1)?.value(&mut cx);
    
    match get_or_create_store(&path) {
        Ok(store) => {
            match store.get_pattern(&pattern) {
                Ok(results) => {
                    let obj = cx.empty_object();
                    for (key, value) in results {
                        let js_value = cx.string(value);
                        obj.set(&mut cx, &*key, js_value)?;
                    }
                    Ok(obj)
                }
                Err(e) => cx.throw_error(format!("Failed to get pattern: {}", e))
            }
        }
        Err(e) => cx.throw_error(e)
    }
}

fn get_range_values(mut cx: FunctionContext) -> JsResult<JsObject> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    let start = cx.argument::<JsString>(1)?.value(&mut cx);
    let end = cx.argument::<JsString>(2)?.value(&mut cx);
    
    match get_or_create_store(&path) {
        Ok(store) => {
            match store.get_range(&start, &end) {
                Ok(results) => {
                    let obj = cx.empty_object();
                    for (key, value) in results {
                        let js_value = cx.string(value);
                        obj.set(&mut cx, &*key, js_value)?;
                    }
                    Ok(obj)
                }
                Err(e) => cx.throw_error(format!("Failed to get range: {}", e))
            }
        }
        Err(e) => cx.throw_error(e)
    }
}

fn flush_store(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    
    match get_or_create_store(&path) {
        Ok(store) => {
            match store.flush() {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("Failed to flush: {}", e))
            }
        }
        Err(e) => cx.throw_error(e)
    }
}

// New function to close a store and remove from cache
fn close_store(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    
    match STORE_CACHE.lock() {
        Ok(mut cache) => {
            cache.remove(&path);
            Ok(cx.undefined())
        }
        Err(e) => cx.throw_error(format!("Failed to close store: {}", e))
    }
}

// New function to clear all cached stores
fn clear_cache(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    match STORE_CACHE.lock() {
        Ok(mut cache) => {
            cache.clear();
            Ok(cx.undefined())
        }
        Err(e) => cx.throw_error(format!("Failed to clear cache: {}", e))
    }
}

fn json_to_js<'a>(cx: &mut FunctionContext<'a>, value: &serde_json::Value) -> JsResult<'a, JsValue> {
    match value {
        serde_json::Value::Null => Ok(cx.null().upcast()),
        serde_json::Value::Bool(b) => Ok(cx.boolean(*b).upcast()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(cx.number(i as f64).upcast())
            } else if let Some(f) = n.as_f64() {
                Ok(cx.number(f).upcast())
            } else {
                Ok(cx.null().upcast())
            }
        }
        serde_json::Value::String(s) => Ok(cx.string(s).upcast()),
        serde_json::Value::Array(arr) => {
            let js_array = cx.empty_array();
            for (i, item) in arr.iter().enumerate() {
                let js_item = json_to_js(cx, item)?;
                js_array.set(cx, i as u32, js_item)?;
            }
            Ok(js_array.upcast())
        }
        serde_json::Value::Object(map) => {
            let js_obj = cx.empty_object();
            for (key, val) in map {
                let js_val = json_to_js(cx, val)?;
                js_obj.set(cx, &**key, js_val)?;
            }
            Ok(js_obj.upcast())
        }
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("open", open_store)?;
    cx.export_function("set", set_value)?;
    cx.export_function("get", get_value)?;
    cx.export_function("delete", delete_key)?;
    cx.export_function("getPattern", get_pattern_matches)?;
    cx.export_function("getRange", get_range_values)?;
    cx.export_function("flush", flush_store)?;
    cx.export_function("close", close_store)?;
    cx.export_function("clearCache", clear_cache)?;
    Ok(())
}