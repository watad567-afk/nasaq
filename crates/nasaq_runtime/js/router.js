/** Nasaq client-side hash router (MVP). */

export function createRouter(routes) {
  const match = () => {
    const hash = location.hash.replace(/^#/, "") || "/";
    const handler = routes[hash] ?? routes["*"];
    if (typeof handler === "function") handler({ path: hash });
  };
  window.addEventListener("hashchange", match);
  match();
  return { navigate: (path) => { location.hash = path; } };
}

export function link(path, label) {
  const a = document.createElement("a");
  a.href = `#${path}`;
  a.textContent = label;
  return a;
}
