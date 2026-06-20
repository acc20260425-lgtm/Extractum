import type { Page } from "@playwright/test";

export function redactUrl(url: string): string {
  return url.replace(/([?&](?:token|key|session|authuser|at|credential)=)[^&]+/gi, "$1<redacted>");
}

export async function reducedDomSnapshot(page: Page): Promise<string> {
  return await page.evaluate(() => {
    const safeAttributes = new Set(["role", "type", "data-testid", "aria-hidden"]);
    const serialize = (element: Element, depth = 0): string => {
      if (depth > 8) return "";
      if (element.matches("script, style, noscript")) return "";
      const attrs = Array.from(element.attributes)
        .filter((attribute) => safeAttributes.has(attribute.name))
        .map((attribute) => ` ${attribute.name}="${attribute.value.replaceAll('"', "&quot;")}"`)
        .join("");
      const children = Array.from(element.children)
        .map((child) => serialize(child, depth + 1))
        .join("");
      const tagName = element.tagName.toLowerCase();
      return `<${tagName}${attrs}>${children}</${tagName}>`;
    };

    return serialize(document.body).slice(0, 200_000);
  }).catch(() => "");
}
