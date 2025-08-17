use neon::prelude::*;
use neon::types::buffer::TypedArray;
use std::sync::Arc;
use std::path::Path;

// Include the core directly (no async wrapper)
mod store {
    include!("../../../waldb.rs");
}

use store::Store;

// Wrapper struct that can be stored in JavaScript
struct StoreWrapper {
    store: Arc<Store>,
}

// Implement Finalize for cleanup when JS object is GC'd
impl Finalize for StoreWrapper {}

// Type alias for convenience
type BoxedStore = JsBox<StoreWrapper>;

// Open database - returns promise with boxed store
fn open(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    std::thread::spawn(move || {
        let result = Store::open(Path::new(&path));
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(store) => {
                    let wrapper = StoreWrapper {
                        store: Arc::new(store),
                    };
                    Ok(cx.boxed(wrapper))
                }
                Err(e) => cx.throw_error(format!("Failed to open store: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Get entries - returns array of [key, value] pairs
fn get_entries(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let prefix = cx.argument::<JsString>(1)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        // Check for exact match first
        if let Ok(Some(value)) = store_arc.get(&prefix) {
            deferred.settle_with(&channel, move |mut cx| {
                let js_array = cx.empty_array();
                let pair = cx.empty_array();
                let js_key = cx.string(prefix);
                let js_value = cx.string(value);
                pair.set(&mut cx, 0, js_key)?;
                pair.set(&mut cx, 1, js_value)?;
                js_array.set(&mut cx, 0, pair)?;
                Ok(js_array)
            });
            return;
        }
        
        // Use get_pattern with wildcard for prefix matching
        let pattern = if prefix.is_empty() {
            "*".to_string()
        } else if prefix.ends_with('/') {
            format!("{}*", prefix)
        } else {
            // No exact match, look for children
            format!("{}/*", prefix)
        };
        
        let result = store_arc.get_pattern(&pattern);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(entries) => {
                    let js_array = cx.empty_array();
                    for (i, (k, v)) in entries.into_iter().enumerate() {
                        let pair = cx.empty_array();
                        let js_key = cx.string(k);
                        let js_value = cx.string(v);
                        pair.set(&mut cx, 0, js_key)?;
                        pair.set(&mut cx, 1, js_value)?;
                        js_array.set(&mut cx, i as u32, pair)?;
                    }
                    Ok(js_array)
                }
                Err(e) => cx.throw_error(format!("Get failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Set value - returns promise
fn set(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    let value = cx.argument::<JsString>(2)?.value(&mut cx);
    let force = cx.argument_opt(3)
        .and_then(|arg| arg.downcast::<JsBoolean, _>(&mut cx).ok())
        .map(|b| b.value(&mut cx))
        .unwrap_or(false);
    
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.set(&key, &value, force);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("Set failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Delete - returns promise
fn delete(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        // Delete key and subtree for Firebase compat
        let _ = store_arc.delete(&key);
        let result = store_arc.delete_subtree(&key);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("Delete failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Set many - returns promise
fn set_many(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let entries_obj = cx.argument::<JsObject>(1)?;
    let replace_subtree_at = cx.argument_opt(2)
        .and_then(|arg| arg.downcast::<JsString, _>(&mut cx).ok())
        .map(|s| s.value(&mut cx));
    
    // Convert JS object to Vec<(String, String)>
    let entries = {
        let keys = entries_obj.get_own_property_names(&mut cx)?;
        let mut entries = Vec::new();
        
        for i in 0..keys.len(&mut cx) {
            let key: Handle<JsString> = keys.get(&mut cx, i)?;
            let key_str = key.value(&mut cx);
            let value: Handle<JsValue> = entries_obj.get(&mut cx, key)?;
            let value: Handle<JsString> = value.downcast_or_throw(&mut cx)?;
            let value_str = value.value(&mut cx);
            entries.push((key_str, value_str));
        }
        
        entries
    };
    
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.set_many(entries, replace_subtree_at.as_deref());
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("SetMany failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Flush - returns promise
fn flush(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.flush();
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("Flush failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Get pattern - returns promise
fn get_pattern(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let pattern = cx.argument::<JsString>(1)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.get_pattern(&pattern);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(matches) => {
                    let obj = cx.empty_object();
                    for (key, value) in matches {
                        let js_key = cx.string(key);
                        let js_value = cx.string(value);
                        obj.set(&mut cx, js_key, js_value)?;
                    }
                    Ok(obj)
                }
                Err(e) => cx.throw_error(format!("GetPattern failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Get range - returns promise
fn get_range(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let start = cx.argument::<JsString>(1)?.value(&mut cx);
    let end = cx.argument::<JsString>(2)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.get_range(&start, &end);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(matches) => {
                    let obj = cx.empty_object();
                    for (key, value) in matches {
                        let js_key = cx.string(key);
                        let js_value = cx.string(value);
                        obj.set(&mut cx, js_key, js_value)?;
                    }
                    Ok(obj)
                }
                Err(e) => cx.throw_error(format!("GetRange failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Get pattern entries - returns array of [key, value] pairs
fn get_pattern_entries(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let pattern = cx.argument::<JsString>(1)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.get_pattern(&pattern);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(matches) => {
                    let js_array = cx.empty_array();
                    for (i, (key, value)) in matches.into_iter().enumerate() {
                        let pair = cx.empty_array();
                        let js_key = cx.string(key);
                        let js_value = cx.string(value);
                        pair.set(&mut cx, 0, js_key)?;
                        pair.set(&mut cx, 1, js_value)?;
                        js_array.set(&mut cx, i as u32, pair)?;
                    }
                    Ok(js_array)
                }
                Err(e) => cx.throw_error(format!("GetPatternEntries failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Get range entries - returns array of [key, value] pairs
fn get_range_entries(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let start = cx.argument::<JsString>(1)?.value(&mut cx);
    let end = cx.argument::<JsString>(2)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.get_range(&start, &end);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(matches) => {
                    let js_array = cx.empty_array();
                    for (i, (key, value)) in matches.into_iter().enumerate() {
                        let pair = cx.empty_array();
                        let js_key = cx.string(key);
                        let js_value = cx.string(value);
                        pair.set(&mut cx, 0, js_key)?;
                        pair.set(&mut cx, 1, js_value)?;
                        js_array.set(&mut cx, i as u32, pair)?;
                    }
                    Ok(js_array)
                }
                Err(e) => cx.throw_error(format!("GetRangeEntries failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// File operations
fn set_file(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let path = cx.argument::<JsString>(1)?.value(&mut cx);
    let buffer = cx.argument::<JsBuffer>(2)?;
    
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    // Get buffer data as bytes
    let data = buffer.as_slice(&mut cx).to_vec();
    
    std::thread::spawn(move || {
        let result = store_arc.set_file(&path, &data);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("SetFile failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

fn get_file(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let path = cx.argument::<JsString>(1)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.get_file(&path);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(data) => {
                    let mut buffer = cx.buffer(data.len())?;
                    let slice = buffer.as_mut_slice(&mut cx);
                    slice.copy_from_slice(&data);
                    Ok(buffer)
                }
                Err(e) => cx.throw_error(format!("GetFile failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

fn delete_file(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let path = cx.argument::<JsString>(1)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.delete_file(&path);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("DeleteFile failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Search operation
fn search(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let pattern = cx.argument::<JsString>(1)?.value(&mut cx);
    let filters_array = cx.argument::<JsArray>(2)?;
    let limit = cx.argument::<JsNumber>(3)?.value(&mut cx) as usize;
    
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    // Parse filters from JS array
    let mut filters = Vec::new();
    for i in 0..filters_array.len(&mut cx) {
        let filter_obj: Handle<JsObject> = filters_array.get(&mut cx, i)?;
        
        let field: Handle<JsString> = filter_obj.get(&mut cx, "field")?;
        let op_str: Handle<JsString> = filter_obj.get(&mut cx, "op")?;
        let value: Handle<JsString> = filter_obj.get(&mut cx, "value")?;
        
        let op = match op_str.value(&mut cx).as_str() {
            "==" => store::FilterOp::Eq,
            "!=" => store::FilterOp::Ne,
            ">" => store::FilterOp::Gt,
            ">=" => store::FilterOp::Gte,
            "<" => store::FilterOp::Lt,
            "<=" => store::FilterOp::Lte,
            _ => store::FilterOp::Eq,
        };
        
        filters.push(store::SearchFilter {
            field: field.value(&mut cx),
            op,
            value: value.value(&mut cx),
        });
    }
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let search_options = store::SearchOptions {
            pattern,
            filters: Some(filters),
            vector: None,
            text: None,
            scoring: None,
            limit: Some(limit),
        };
        let result = store_arc.search(search_options);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(groups) => {
                    let js_array = cx.empty_array();
                    
                    for (idx, (group_key, fields)) in groups.into_iter().enumerate() {
                        let group_array = cx.empty_array();
                        
                        // Add all fields as [key, value] pairs
                        let mut field_idx = 0;
                        for (field_name, field_value) in fields {
                            let pair = cx.empty_array();
                            let full_key = format!("{}/{}", group_key, field_name);
                            let js_key = cx.string(full_key);
                            let js_value = cx.string(field_value);
                            pair.set(&mut cx, 0, js_key)?;
                            pair.set(&mut cx, 1, js_value)?;
                            group_array.set(&mut cx, field_idx, pair)?;
                            field_idx += 1;
                        }
                        
                        js_array.set(&mut cx, idx as u32, group_array)?;
                    }
                    
                    Ok(js_array)
                }
                Err(e) => cx.throw_error(format!("Search failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Set vector embedding
fn set_vector(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let path = cx.argument::<JsString>(1)?.value(&mut cx);
    let vector_array = cx.argument::<JsArray>(2)?;
    
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    // Convert JS array to Vec<f32>
    let mut vector = Vec::new();
    for i in 0..vector_array.len(&mut cx) {
        let val: Handle<JsNumber> = vector_array.get(&mut cx, i)?;
        vector.push(val.value(&mut cx) as f32);
    }
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.set_vector(&path, vector);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(format!("Failed to set vector: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Get vector embedding
fn get_vector(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let path = cx.argument::<JsString>(1)?.value(&mut cx);
    
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.get_vector(&path);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(Some(vector)) => {
                    let js_array = cx.empty_array();
                    for (i, &val) in vector.iter().enumerate() {
                        let js_val = cx.number(val);
                        js_array.set(&mut cx, i as u32, js_val)?;
                    }
                    Ok(js_array.upcast::<JsValue>())
                }
                Ok(None) => Ok(cx.null().upcast::<JsValue>()),
                Err(e) => cx.throw_error(format!("Failed to get vector: {}", e))
            }
        });
    });
    
    Ok(promise)
}

// Advanced search with vector/text search
fn advanced_search(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedStore>(0)?;
    let options = cx.argument::<JsObject>(1)?;
    
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    // Parse search options
    let pattern: Handle<JsString> = options.get(&mut cx, "pattern")?;
    let pattern = pattern.value(&mut cx);
    
    // Parse basic filters
    let mut filters = None;
    if let Ok(filters_value) = options.get::<JsValue, _, _>(&mut cx, "filters") {
        if !filters_value.is_a::<JsUndefined, _>(&mut cx) && !filters_value.is_a::<JsNull, _>(&mut cx) {
            let filters_array = filters_value.downcast::<JsArray, _>(&mut cx).or_throw(&mut cx)?;
        let mut parsed_filters = Vec::new();
        for i in 0..filters_array.len(&mut cx) {
            let filter: Handle<JsObject> = filters_array.get(&mut cx, i)?;
            
            let field: Handle<JsString> = filter.get(&mut cx, "field")?;
            let op: Handle<JsString> = filter.get(&mut cx, "op")?;
            let value: Handle<JsString> = filter.get(&mut cx, "value")?;
            
            let op = match op.value(&mut cx).as_str() {
                "==" => store::FilterOp::Eq,
                "!=" => store::FilterOp::Ne,
                ">" => store::FilterOp::Gt,
                ">=" => store::FilterOp::Gte,
                "<" => store::FilterOp::Lt,
                "<=" => store::FilterOp::Lte,
                _ => return cx.throw_error("Invalid filter operator")
            };
            
            parsed_filters.push(store::SearchFilter {
                field: field.value(&mut cx),
                op,
                value: value.value(&mut cx),
            });
        }
            if !parsed_filters.is_empty() {
                filters = Some(parsed_filters);
            }
        }
    }
    
    // Parse vector search options
    let mut vector_opts = None;
    if let Ok(vector_value) = options.get::<JsValue, _, _>(&mut cx, "vector") {
        if !vector_value.is_a::<JsUndefined, _>(&mut cx) && !vector_value.is_a::<JsNull, _>(&mut cx) {
            let vector_obj = vector_value.downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
        let query_array: Handle<JsArray> = vector_obj.get(&mut cx, "query")?;
        let field: Handle<JsString> = vector_obj.get(&mut cx, "field")?;
        
        let mut query = Vec::new();
        for i in 0..query_array.len(&mut cx) {
            let val: Handle<JsNumber> = query_array.get(&mut cx, i)?;
            query.push(val.value(&mut cx) as f32);
        }
        
        let threshold = if let Ok(threshold_value) = vector_obj.get::<JsValue, _, _>(&mut cx, "threshold") {
            if !threshold_value.is_a::<JsUndefined, _>(&mut cx) && !threshold_value.is_a::<JsNull, _>(&mut cx) {
                let threshold_num = threshold_value.downcast::<JsNumber, _>(&mut cx).or_throw(&mut cx)?;
                Some(threshold_num.value(&mut cx) as f32)
            } else {
                None
            }
        } else {
            None
        };
        
            vector_opts = Some(store::VectorSearchOptions {
                query,
                field: field.value(&mut cx),
                threshold,
            });
        }
    }
    
    // Parse text search options
    let mut text_opts = None;
    if let Ok(text_value) = options.get::<JsValue, _, _>(&mut cx, "text") {
        if !text_value.is_a::<JsUndefined, _>(&mut cx) && !text_value.is_a::<JsNull, _>(&mut cx) {
            let text_obj = text_value.downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
        let query: Handle<JsString> = text_obj.get(&mut cx, "query")?;
        let fields_array: Handle<JsArray> = text_obj.get(&mut cx, "fields")?;
        
        let mut fields = Vec::new();
        for i in 0..fields_array.len(&mut cx) {
            let field: Handle<JsString> = fields_array.get(&mut cx, i)?;
            fields.push(field.value(&mut cx));
        }
        
        let case_sensitive = if let Ok(cs_value) = text_obj.get::<JsValue, _, _>(&mut cx, "caseSensitive") {
            if !cs_value.is_a::<JsUndefined, _>(&mut cx) && !cs_value.is_a::<JsNull, _>(&mut cx) {
                let cs_bool = cs_value.downcast::<JsBoolean, _>(&mut cx).or_throw(&mut cx)?;
                Some(cs_bool.value(&mut cx))
            } else {
                None
            }
        } else {
            None
        };
        
            text_opts = Some(store::TextSearchOptions {
                query: query.value(&mut cx),
                fields,
                case_sensitive,
            });
        }
    }
    
    // Parse scoring weights
    let mut scoring = None;
    if let Ok(scoring_value) = options.get::<JsValue, _, _>(&mut cx, "scoring") {
        if !scoring_value.is_a::<JsUndefined, _>(&mut cx) && !scoring_value.is_a::<JsNull, _>(&mut cx) {
            let scoring_obj = scoring_value.downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
        let vector = if let Ok(v_value) = scoring_obj.get::<JsValue, _, _>(&mut cx, "vector") {
            if !v_value.is_a::<JsUndefined, _>(&mut cx) && !v_value.is_a::<JsNull, _>(&mut cx) {
                let v_num = v_value.downcast::<JsNumber, _>(&mut cx).or_throw(&mut cx)?;
                v_num.value(&mut cx) as f32
            } else {
                1.0
            }
        } else {
            1.0
        };
        
        let text = if let Ok(t_value) = scoring_obj.get::<JsValue, _, _>(&mut cx, "text") {
            if !t_value.is_a::<JsUndefined, _>(&mut cx) && !t_value.is_a::<JsNull, _>(&mut cx) {
                let t_num = t_value.downcast::<JsNumber, _>(&mut cx).or_throw(&mut cx)?;
                t_num.value(&mut cx) as f32
            } else {
                1.0
            }
        } else {
            1.0
        };
        
        let filter = if let Ok(f_value) = scoring_obj.get::<JsValue, _, _>(&mut cx, "filter") {
            if !f_value.is_a::<JsUndefined, _>(&mut cx) && !f_value.is_a::<JsNull, _>(&mut cx) {
                let f_num = f_value.downcast::<JsNumber, _>(&mut cx).or_throw(&mut cx)?;
                f_num.value(&mut cx) as f32
            } else {
                1.0
            }
        } else {
            1.0
        };
        
            scoring = Some(store::ScoringWeights { vector, text, filter });
        }
    }
    
    // Parse limit
    let limit = if let Ok(limit_value) = options.get::<JsValue, _, _>(&mut cx, "limit") {
        if !limit_value.is_a::<JsUndefined, _>(&mut cx) && !limit_value.is_a::<JsNull, _>(&mut cx) {
            let limit_num = limit_value.downcast::<JsNumber, _>(&mut cx).or_throw(&mut cx)?;
            Some(limit_num.value(&mut cx) as usize)
        } else {
            None
        }
    } else {
        None
    };
    
    let search_options = store::SearchOptions {
        pattern,
        filters,
        vector: vector_opts,
        text: text_opts,
        scoring,
        limit,
    };
    
    let store_arc = Arc::clone(&store.store);
    
    std::thread::spawn(move || {
        let result = store_arc.search(search_options);
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(groups) => {
                    let js_array = cx.empty_array();
                    
                    for (i, (group_key, fields)) in groups.iter().enumerate() {
                        let group_array = cx.empty_array();
                        
                        for (j, (key, value)) in fields.iter().enumerate() {
                            let entry_array = cx.empty_array();
                            let full_key = if group_key.is_empty() {
                                key.clone()
                            } else {
                                format!("{}/{}", group_key, key)
                            };
                            let js_key = cx.string(full_key);
                            let js_value = cx.string(value);
                            entry_array.set(&mut cx, 0, js_key)?;
                            entry_array.set(&mut cx, 1, js_value)?;
                            group_array.set(&mut cx, j as u32, entry_array)?;
                        }
                        
                        js_array.set(&mut cx, i as u32, group_array)?;
                    }
                    
                    Ok(js_array)
                }
                Err(e) => cx.throw_error(format!("Advanced search failed: {}", e))
            }
        });
    });
    
    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("open", open)?;
    cx.export_function("getEntries", get_entries)?;
    cx.export_function("set", set)?;
    cx.export_function("delete", delete)?;
    cx.export_function("setMany", set_many)?;
    cx.export_function("flush", flush)?;
    cx.export_function("getPattern", get_pattern)?;
    cx.export_function("getRange", get_range)?;
    cx.export_function("getPatternEntries", get_pattern_entries)?;
    cx.export_function("getRangeEntries", get_range_entries)?;
    cx.export_function("setFile", set_file)?;
    cx.export_function("getFile", get_file)?;
    cx.export_function("deleteFile", delete_file)?;
    cx.export_function("search", search)?;
    cx.export_function("setVector", set_vector)?;
    cx.export_function("getVector", get_vector)?;
    cx.export_function("advancedSearch", advanced_search)?;
    
    Ok(())
}