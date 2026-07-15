import { describe, expect, it } from "vitest";

const componentSources = import.meta.glob<string>(
	"./components/research-projects/*.svelte",
	{
		query: "?raw",
		import: "default",
		eager: true,
	},
);

const LUCIDE_IMPORT = /from\s+["'](@lucide\/svelte[^"']*)["']/g;

function normalize(source: string): string {
	return source.replace(/\r\n/g, "\n");
}

function findOffenders(): string[] {
	return Object.entries(componentSources)
		.flatMap(([path, rawSource]) => {
			const source = normalize(rawSource);
			return [...source.matchAll(LUCIDE_IMPORT)]
				.map((match) => match[1])
				.filter((specifier): specifier is string => Boolean(specifier))
				.filter(
					(specifier) => !specifier.startsWith("@lucide/svelte/icons/"),
				)
				.map((specifier) => `${path}: ${specifier}`);
		})
		.sort();
}

describe("research-projects Lucide import boundary", () => {
	it("uses only direct Lucide icon modules", () => {
		const offenders = findOffenders();
		if (offenders.length > 0) {
			console.error(offenders.join("\n"));
		}
		expect(offenders).toEqual([]);
	});
});
