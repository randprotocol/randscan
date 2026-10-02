// A message with a little markup — <a href>, <code>, <strong>, <em>, <br/> — as a token tree
// React can render without innerHTML. Anything that is not one of those tags is text.
const TAG = /<(a)\s+href="([^"]*)">|<(code|strong|em)>|<\/(a|code|strong|em)>|<br\s*\/?>/g;

/** @param {string} s @param {Record<string, string | number>} [vars] */
export function fill(s, vars = {}) {
  return s.replace(/\{(\w+)\}/g, (m, k) => (k in vars ? String(vars[k]) : m));
}

/** @param {string} s */
export function tokenize(s) {
  const root = { children: [] };
  const stack = [root];
  let last = 0;
  const top = () => stack[stack.length - 1];
  const text = (t) => {
    if (t) top().children.push({ type: 'text', text: t });
  };
  for (const m of s.matchAll(TAG)) {
    text(s.slice(last, m.index));
    last = m.index + m[0].length;
    if (m[1]) {
      const node = { type: 'a', href: m[2], children: [] };
      top().children.push(node);
      stack.push(node);
    } else if (m[3]) {
      const node = { type: m[3], children: [] };
      top().children.push(node);
      stack.push(node);
    } else if (m[4]) {
      if (stack.length > 1) stack.pop();
    } else {
      top().children.push({ type: 'br' });
    }
  }
  text(s.slice(last));
  return root.children;
}
