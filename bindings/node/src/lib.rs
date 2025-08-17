use neon::prelude::*;
use std::sync::Arc;
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;

// Include the async wrapper
mod async_store {
    include!("../../../async_wrapper.rs");
}

use async_store::AsyncStore;

// Global tokio runtime for async operations
static RUNTIME: Lazy<Arc<Runtime>> = Lazy::new(|| {
    Arc::new(
        Runtime::new().expect("Failed to create Tokio runtime")
    )
});

// Wrapper struct that can be stored in JavaScript
struct StoreWrapper {
    store: Arc<AsyncStore>,
}

// Implement Finalize for cleanup when JS object is GC'd
impl Finalize for StoreWrapper {}

// Type alias for convenience
type BoxedAsyncStore = JsBox<StoreWrapper>;

// Open database - returns promise with boxed store
fn open(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    RUNTIME.spawn(async move {
        let result = AsyncStore::open(std::path::Path::new(&path)).await;
        
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

// Get value - returns promise
fn get(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedAsyncStore>(0)?;
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    RUNTIME.spawn(async move {
        let result = store_arc.get(&key).await.map_err(|e| e.to_string());
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(Some(value)) => Ok(cx.string(value).upcast::<JsValue>()),
                Ok(None) => Ok(cx.null().upcast::<JsValue>()),
                Err(e) => cx.throw_error(e)
            }
        });
    });
    
    Ok(promise)
}

// Set value - returns promise
fn set(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedAsyncStore>(0)?;
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    let value = cx.argument::<JsString>(2)?.value(&mut cx);
    let force = cx.argument_opt(3)
        .and_then(|arg| arg.downcast::<JsBoolean, _>(&mut cx).ok())
        .map(|b| b.value(&mut cx))
        .unwrap_or(false);
    
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    RUNTIME.spawn(async move {
        let result = store_arc.set(&key, &value, force).await.map_err(|e| e.to_string());
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(e)
            }
        });
    });
    
    Ok(promise)
}

// Delete - returns promise
fn delete(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedAsyncStore>(0)?;
    let key = cx.argument::<JsString>(1)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    RUNTIME.spawn(async move {
        // Delete key and subtree for Firebase compat
        let r1 = store_arc.delete(&key).await;
        let r2 = store_arc.delete_subtree(&key).await;
        let result = r1.and(r2).map_err(|e| e.to_string());
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(e)
            }
        });
    });
    
    Ok(promise)
}

// Set many - returns promise
fn set_many(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedAsyncStore>(0)?;
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
    
    RUNTIME.spawn(async move {
        let result = store_arc.set_many(entries, replace_subtree_at.as_deref()).await.map_err(|e| e.to_string());
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(e)
            }
        });
    });
    
    Ok(promise)
}

// Flush - returns promise
fn flush(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedAsyncStore>(0)?;
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    RUNTIME.spawn(async move {
        let result = store_arc.flush().await.map_err(|e| e.to_string());
        
        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(e)
            }
        });
    });
    
    Ok(promise)
}

// Get pattern - returns promise
fn get_pattern(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedAsyncStore>(0)?;
    let pattern = cx.argument::<JsString>(1)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    RUNTIME.spawn(async move {
        let result = store_arc.get_pattern(&pattern).await.map_err(|e| e.to_string());
        
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
                Err(e) => cx.throw_error(e)
            }
        });
    });
    
    Ok(promise)
}

// Get range - returns promise
fn get_range(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let store = cx.argument::<BoxedAsyncStore>(0)?;
    let start = cx.argument::<JsString>(1)?.value(&mut cx);
    let end = cx.argument::<JsString>(2)?.value(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    
    let store_arc = Arc::clone(&store.store);
    
    RUNTIME.spawn(async move {
        let result = store_arc.get_range(&start, &end).await.map_err(|e| e.to_string());
        
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
                Err(e) => cx.throw_error(e)
            }
        });
    });
    
    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // All methods return promises - true async!
    cx.export_function("open", open)?;
    cx.export_function("get", get)?;
    cx.export_function("set", set)?;
    cx.export_function("delete", delete)?;
    cx.export_function("setMany", set_many)?;
    cx.export_function("flush", flush)?;
    cx.export_function("getPattern", get_pattern)?;
    cx.export_function("getRange", get_range)?;
    
    Ok(())
}