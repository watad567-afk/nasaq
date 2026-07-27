/**
 * Nasaq official runtime — minimal core helpers for Phase 1.
 * This is NOT React, Vue, or any third-party UI framework.
 */

export function println(value) {
  console.log(value);
}

export function println_int(value) {
  console.log(value);
}

export function println_str(value) {
  console.log(value);
}

export function assert_eq(left, right) {
  if (left !== right) {
    throw new Error(`assertion failed: ${left} !== ${right}`);
  }
}

export function identity(value) {
  return value;
}

/** Host stub for std/string.nq */
export function __str_len(s) {
  return String(s).length;
}

/** Host stubs for registry json package */
export function __json_stringify(value) {
  return JSON.stringify(value);
}

export function __json_parse(text) {
  return JSON.stringify(JSON.parse(text));
}
