/** Nasaq Web — fine-grained signal runtime (no Virtual DOM). */

let activeEffect = null;

export function createSignal(initial) {
  let value = initial;
  const subs = new Set();
  return {
    get() {
      if (activeEffect) subs.add(activeEffect);
      return value;
    },
    set(next) {
      if (Object.is(next, value)) return;
      value = next;
      for (const fn of subs) fn();
    },
    subscribe(fn) {
      subs.add(fn);
      return () => subs.delete(fn);
    },
  };
}

export function effect(fn) {
  const run = () => {
    activeEffect = run;
    try {
      fn();
    } finally {
      activeEffect = null;
    }
  };
  run();
  return run;
}

function resolveFactory(factory) {
  const comp = factory();
  if (typeof comp === "function") {
    return { mount: comp, hydrate: null, clickHandlers: [] };
  }
  return {
    mount: comp.mount,
    hydrate: comp.hydrate,
    clickHandlers: comp.clickHandlers || [],
    signals: comp.signals || {},
  };
}

export function mountComponent(factory, selector) {
  const root = document.querySelector(selector);
  if (!root) throw new Error(`mount target not found: ${selector}`);
  root.replaceChildren();
  const comp = resolveFactory(factory);
  comp.mount(root);
}

export function hydrateComponent(factory, selector) {
  const root = document.querySelector(selector);
  if (!root) throw new Error(`mount target not found: ${selector}`);
  const comp = resolveFactory(factory);
  if (comp.hydrate) {
    comp.hydrate(root);
    return;
  }
  wireHydration(root, comp);
}

function wireHydration(root, comp) {
  const signals = comp.signals || {};
  root.querySelectorAll("[data-nasaq-signal]").forEach((el) => {
    const name = el.getAttribute("data-nasaq-signal");
    const signal = signals[name];
    if (!signal) return;
    el.textContent = String(signal.get());
    effect(() => {
      el.textContent = String(signal.get());
    });
  });
  root.querySelectorAll("[data-nasaq-click]").forEach((el) => {
    const idx = Number(el.getAttribute("data-nasaq-click"));
    const eventName = el.getAttribute("data-nasaq-event") || "click";
    const handler = comp.clickHandlers[idx];
    if (handler) {
      el.addEventListener(eventName, handler);
    }
  });
}
