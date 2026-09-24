/// boa JavaScript engine https://github.com/boa-dev/boa
/// Native Messaging host (nm_boa.js)
/// Globals defined for I/O in ECMAScript: std.read(), std.write(), std.err()
/// guest271314 9-23-2026

use boa_engine::object::builtins::{JsArrayBuffer, JsUint8Array};
use boa_engine::object::FunctionObjectBuilder;
use boa_engine::{js_string, Context, JsObject, JsResult, JsValue, NativeFunction, Source};
use std::io::{self, Read, Write};

/// High-performance synchronous block parsing from stdin (Works on native and WASI)
fn read_bytes(bytes_to_read: usize) -> Option<Vec<u8>> {
  if bytes_to_read == 0 {
    return Some(vec![]);
  }

  let mut buffer = vec![0u8; bytes_to_read];
  let mut stdin = io::stdin().lock();

  // 💡 FIX: Read whatever bytes are available up to capacity (do not use read_exact)
  match stdin.read(&mut buffer) {
    Ok(0) => None, // Clean EOF / Stream closed
    Ok(n) => {
      buffer.truncate(n);
      Some(buffer)
    }
    Err(_) => None,
  }
}

/// Dispatches binary blocks cleanly back across stdout and returns amount written
fn write_stdout(bytes: &[u8]) -> Option<usize> {
  let mut stdout = io::stdout().lock();
  if stdout.write_all(bytes).is_err() {
    return None;
  }
  if stdout.flush().is_err() {
    return None;
  }
  Some(bytes.len())
}

/// Forwards standard error diagnostics safely without corrupting the stdout pipeline
fn write_stderr(message: &str) {
  let mut stderr = io::stderr().lock();
  let _ = stderr.write_all(message.as_bytes());
  let _ = stderr.flush();
}

fn main() -> JsResult<()> {
  // DIRECTLY REFERENCE THE JS FILE IN MAIN.RS
  let js_code = include_str!("index.js");

  // Initialize execution context
  let mut context = Context::default();

  // Build the top-level 'std' namespace object template
  let std_obj = JsObject::with_object_proto(context.intrinsics());

  // Inject std.read(typedArray) -> returns bytes read or null
  let read_fn = NativeFunction::from_copy_closure(|_this, args, ctx| {
    let js_buffer = args.get(0).and_then(|v| v.as_object().cloned());

    if let Some(obj) = js_buffer {
      if let Ok(typed_array) = JsUint8Array::from_object(obj) {
        let len = typed_array.length(ctx).unwrap_or(0);
        let byte_offset = typed_array.byte_offset(ctx).unwrap_or(0);

        if let Some(bytes) = read_bytes(len) {
          let bytes_read = bytes.len();

          let ab_val = typed_array.buffer(ctx)?;
          if let Some(ab_obj) = ab_val.as_object().cloned() {
            let ab = JsArrayBuffer::from_object(ab_obj)?;

            let write_result = if let Some(mut gc_ref_mut) = ab.data_mut() {
              let slice = &mut *gc_ref_mut;
              slice[byte_offset..(byte_offset + bytes_read)].copy_from_slice(&bytes[..bytes_read]);
              Some(JsValue::from(bytes_read))
            } else {
              None
            };

            if let Some(val) = write_result {
              return Ok(val);
            }
          }
        }
      }
    }
    Ok(JsValue::null())
  });
  let read_js_fn = FunctionObjectBuilder::new(context.realm(), read_fn).build();
  std_obj.set(
    js_string!("read"),
    JsValue::from(read_js_fn),
    true,
    &mut context,
  )?;

  // Inject std.write(typedArray) -> returns bytes written or 0
  let write_fn = NativeFunction::from_copy_closure(|_this, args, ctx| {
    let js_buffer = args.get(0).and_then(|v| v.as_object().cloned());
    if let Some(obj) = js_buffer {
      if let Ok(typed_array) = JsUint8Array::from_object(obj) {
        let byte_offset = typed_array.byte_offset(ctx).unwrap_or(0);
        let length = typed_array.length(ctx).unwrap_or(0);

        let array_buffer_val = typed_array.buffer(ctx)?;
        if let Some(array_buffer_obj) = array_buffer_val.as_object().cloned() {
          let array_buffer = JsArrayBuffer::from_object(array_buffer_obj)?;

          let write_result = if let Some(gc_ref) = array_buffer.data() {
            let bytes: &[u8] = &*gc_ref;
            let view_window_slice = &bytes[byte_offset..(byte_offset + length)];
            write_stdout(view_window_slice).map(JsValue::from)
          } else {
            None
          };

          if let Some(val) = write_result {
            return Ok(val);
          }
        }
      }
    }
    Ok(JsValue::from(0))
  });
  let write_js_fn = FunctionObjectBuilder::new(context.realm(), write_fn).build();
  std_obj.set(
    js_string!("write"),
    JsValue::from(write_js_fn),
    true,
    &mut context,
  )?;

  // Inject std.err(string)
  let err_fn = NativeFunction::from_copy_closure(|_this, args, _ctx| {
    let js_str = args
      .get(0)
      .and_then(|v| v.as_string())
      .cloned()
      .unwrap_or_default();
    write_stderr(&js_str.to_std_string_escaped());
    Ok(JsValue::undefined())
  });
  let err_js_fn = FunctionObjectBuilder::new(context.realm(), err_fn).build();
  std_obj.set(
    js_string!("err"),
    JsValue::from(err_js_fn),
    true,
    &mut context,
  )?;

  // Register 'std' globally
  context.global_object().set(
    js_string!("std"),
    JsValue::from(std_obj),
    true,
    &mut context,
  )?;

  // Parse execution with explicit error tracking to stderr if JavaScript crashes
  match context.eval(Source::from_bytes(js_code.as_bytes())) {
    Ok(_) => {
      context.run_jobs();
    }
    Err(js_error) => {
      eprintln!("JavaScript Exception: {}", js_error);
    }
  }

  Ok(())
}
