import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const migrationRegistryPath = "src-tauri/src/migrations.rs";
const fixtureCandidates = [
  "src-tauri/src/analysis/test_schema.rs",
  "src-tauri/crates/extractum-analysis/src/test_schema.rs",
] as const;

const normalize = (value: string) => value.replace(/\r\n/g, "\n");
const read = (relativePath: string) =>
  normalize(readFileSync(path.join(repositoryRoot, relativePath), "utf8"));
const toRepositoryPath = (absolutePath: string) =>
  path.relative(repositoryRoot, absolutePath).replaceAll("\\", "/");
const occurrences = (source: string, pattern: RegExp) =>
  [
    ...source.matchAll(
      new RegExp(
        pattern.source,
        pattern.flags.includes("g") ? pattern.flags : `${pattern.flags}g`,
      ),
    ),
  ].length;

const matchingDelimiter = (
  source: string,
  openIndex: number,
  open: string,
  close: string,
) => {
  let depth = 0;
  for (let index = openIndex; index < source.length; index += 1) {
    if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      index = newline < 0 ? source.length : newline;
      continue;
    }
    if (source.startsWith("/*", index)) {
      let commentDepth = 1;
      index += 2;
      while (index < source.length && commentDepth > 0) {
        if (source.startsWith("/*", index)) {
          commentDepth += 1;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          commentDepth -= 1;
          index += 2;
        } else {
          index += 1;
        }
      }
      index -= 1;
      continue;
    }
    const rawString = source.slice(index).match(/^(?:b)?r(#{0,255})"/);
    if (rawString) {
      const closing = `"${rawString[1]}`;
      const end = source.indexOf(closing, index + rawString[0].length);
      if (end < 0) throw new Error("unclosed Rust raw string literal");
      index = end + closing.length - 1;
      continue;
    }
    const quoteIndex =
      source[index] === '"' ? index : source.startsWith('b"', index) ? index + 1 : -1;
    if (quoteIndex >= 0) {
      index = quoteIndex + 1;
      while (index < source.length) {
        if (source[index] === "\\") index += 2;
        else if (source[index] === '"') break;
        else index += 1;
      }
      if (index >= source.length) throw new Error("unclosed Rust string literal");
      continue;
    }
    if (source[index] === open) depth += 1;
    if (source[index] === close) depth -= 1;
    if (depth === 0) return index;
  }
  throw new Error(`unclosed ${open}${close} delimiter`);
};

const rustBlock = (source: string, marker: string) => {
  const start = source.indexOf(marker);
  if (start < 0) throw new Error(`missing Rust block marker: ${marker}`);
  const open = source.indexOf("{", start);
  if (open < 0) throw new Error(`missing Rust block opening brace: ${marker}`);
  return source.slice(start, matchingDelimiter(source, open, "{", "}") + 1);
};

const parseMigrationRegistry = (source: string) => {
  const buildMarker = "pub fn build_migrations() -> Vec<Migration>";
  const buildMatches = [
    ...source.matchAll(/^pub fn build_migrations\(\) -> Vec<Migration>\s*\{/gm),
  ];
  if (buildMatches.length !== 1 || buildMatches[0].index === undefined) {
    throw new Error(
      `expected exactly one build_migrations() registry, found ${buildMatches.length}`,
    );
  }
  const build = rustBlock(source.slice(buildMatches[0].index), buildMarker);
  const vectorMarker = "let mut migrations = vec![";
  const vectorStart = build.indexOf(vectorMarker);
  const extendMarker = "migrations.extend(apalis_sqlite_migrations());";
  const extendStarts = [...build.matchAll(/migrations\.extend\(apalis_sqlite_migrations\(\)\);/g)]
    .map((match) => match.index)
    .filter((index): index is number => index !== undefined);
  const extendStart = extendStarts[0] ?? -1;
  if (vectorStart < 0 || extendStart < 0) {
    throw new Error("unparseable build_migrations() non-Apalis prefix");
  }
  const buildOpen = build.indexOf("{");
  if (build.slice(buildOpen + 1, vectorStart).trim() !== "") {
    throw new Error("unmatched token before the migration vector");
  }
  if (extendStarts.length !== 1) {
    throw new Error(
      `expected one Apalis extension in build_migrations(), found ${extendStarts.length}`,
    );
  }
  const vectorOpen = vectorStart + vectorMarker.length - 1;
  const vectorClose = matchingDelimiter(build, vectorOpen, "[", "]");
  if (build.slice(vectorClose, vectorClose + 2) !== "];") {
    throw new Error("missing migration vector closing ];");
  }
  if (extendStart <= vectorClose) {
    throw new Error("Apalis extension precedes the non-Apalis migration vector");
  }
  if (build.slice(vectorClose + 2, extendStart).trim() !== "") {
    throw new Error("unmatched token before the Apalis migration extension");
  }
  const buildClose = build.lastIndexOf("}");
  if (build.slice(extendStart + extendMarker.length, buildClose).trim() !== "migrations") {
    throw new Error("unmatched token after the Apalis migration extension");
  }
  const vectorBody = build.slice(vectorOpen + 1, vectorClose);
  const calls = [...vectorBody.matchAll(/^\s*([a-z][a-z0-9_]*)\(\),\s*$/gm)].map(
    (match) => match[1],
  );
  if (calls.length === 0) throw new Error("empty build_migrations() non-Apalis prefix");
  if (new Set(calls).size !== calls.length) {
    throw new Error("duplicate migration call in build_migrations() prefix");
  }
  if (vectorBody.replace(/^\s*[a-z][a-z0-9_]*\(\),\s*$/gm, "").trim() !== "") {
    throw new Error("unmatched token in build_migrations() prefix");
  }

  return calls.map((functionName) => {
    const matches = [
      ...source.matchAll(new RegExp(`^fn ${functionName}\\(\\) -> Migration \\{`, "gm")),
    ];
    if (matches.length !== 1 || matches[0].index === undefined) {
      throw new Error(`expected one migration function ${functionName}`);
    }
    const body = rustBlock(source.slice(matches[0].index), `fn ${functionName}()`);
    const sqlTokens = [...body.matchAll(/^\s*sql:\s*([A-Z][A-Z0-9_]+),\s*$/gm)].map(
      (match) => match[1],
    );
    if (sqlTokens.length !== 1) {
      throw new Error(`expected one sql token in ${functionName}`);
    }
    const constantPattern = new RegExp(
      `^const ${sqlTokens[0]}: &str =\\s*include_str!\\("([^"]+)"\\);$`,
      "gm",
    );
    const constants = [...source.matchAll(constantPattern)];
    if (constants.length !== 1) {
      throw new Error(`expected one include_str! constant ${sqlTokens[0]}`);
    }
    return constants[0][1];
  });
};

type FixtureMigration = {
  repositoryPath: string;
  includePath: string;
};

const parseFixtureMigrations = (source: string): FixtureMigration[] => {
  if (
    /\bpub(?:\s*\([^)]*\))?\s+const\s+ANALYSIS_TEST_MIGRATIONS\b/.test(source)
  ) {
    throw new Error("ANALYSIS_TEST_MIGRATIONS must remain private");
  }
  const declaration =
    /const ANALYSIS_TEST_MIGRATIONS:\s*\[\(&str, &str\);\s*(\d+)\]\s*=\s*\[/m.exec(
      source,
    );
  if (!declaration || declaration.index === undefined) {
    throw new Error("unparseable ANALYSIS_TEST_MIGRATIONS declaration");
  }
  const declaredCount = Number(declaration[1]);
  const bodyOpen = declaration.index + declaration[0].lastIndexOf("[");
  const bodyClose = matchingDelimiter(source, bodyOpen, "[", "]");
  if (source.slice(bodyClose, bodyClose + 2) !== "];") {
    throw new Error("missing ANALYSIS_TEST_MIGRATIONS closing ];");
  }
  const body = source.slice(bodyOpen + 1, bodyClose);
  const pairPattern =
    /\(\s*"([^"]+)"\s*,\s*include_str!\(\s*"([^"]+)"\s*\)\s*,?\s*\)\s*,?/g;
  const pairs = [...body.matchAll(pairPattern)].map((match) => ({
    repositoryPath: match[1],
    includePath: match[2],
  }));
  if (pairs.length !== declaredCount) {
    throw new Error(
      `fixture declares ${declaredCount} migrations but parsed ${pairs.length}`,
    );
  }
  if (body.replace(pairPattern, "").trim() !== "") {
    throw new Error("unmatched token in ANALYSIS_TEST_MIGRATIONS");
  }
  if (new Set(pairs.map((pair) => pair.repositoryPath)).size !== pairs.length) {
    throw new Error("duplicate repository path in ANALYSIS_TEST_MIGRATIONS");
  }
  if (new Set(pairs.map((pair) => pair.includePath)).size !== pairs.length) {
    throw new Error("duplicate include path in ANALYSIS_TEST_MIGRATIONS");
  }
  if (occurrences(source, /include_str!\s*\(/g) !== pairs.length) {
    throw new Error("unexpected include_str! outside ANALYSIS_TEST_MIGRATIONS");
  }
  return pairs;
};

const maskRustNonCode = (source: string) => {
  const masked = source.split("");
  const blank = (start: number, end: number) => {
    for (let index = start; index < end; index += 1) {
      if (masked[index] !== "\n") masked[index] = " ";
    }
  };
  for (let index = 0; index < source.length;) {
    if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      const end = newline < 0 ? source.length : newline;
      blank(index, end);
      index = end;
      continue;
    }
    if (source.startsWith("/*", index)) {
      const start = index;
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth += 1;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth -= 1;
          index += 2;
        } else {
          index += 1;
        }
      }
      blank(start, index);
      continue;
    }
    const raw = source.slice(index).match(/^(?:b)?r(#{0,255})"/);
    if (raw) {
      const closing = `"${raw[1]}`;
      const end = source.indexOf(closing, index + raw[0].length);
      if (end < 0) throw new Error("unclosed Rust raw string literal");
      const tokenEnd = end + closing.length;
      blank(index, tokenEnd);
      index = tokenEnd;
      continue;
    }
    const quote = source[index] === '"' ? index : source.startsWith('b"', index) ? index + 1 : -1;
    if (quote >= 0) {
      const start = index;
      index = quote + 1;
      while (index < source.length) {
        if (source[index] === "\\") index += 2;
        else if (source[index] === '"') {
          index += 1;
          break;
        } else index += 1;
      }
      blank(start, index);
      continue;
    }
    index += 1;
  }
  return masked.join("");
};

const rustFunctionContaining = (source: string, offset: number) => {
  const syntax = maskRustNonCode(source);
  const functions = [...syntax.matchAll(
    /\b(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{}]*>)?\s*\(/g,
  )].flatMap((match) => {
    if (match.index === undefined) return [];
    const paramsOpen = match.index + match[0].lastIndexOf("(");
    const paramsClose = matchingDelimiter(syntax, paramsOpen, "(", ")");
    const bodyOpen = syntax.indexOf("{", paramsClose);
    const declarationEnd = syntax.indexOf(";", paramsClose);
    if (bodyOpen < 0 || (declarationEnd >= 0 && declarationEnd < bodyOpen)) return [];
    const bodyClose = matchingDelimiter(syntax, bodyOpen, "{", "}");
    return [{
      name: match[1],
      start: match.index,
      bodyOpen,
      bodyClose,
      source: source.slice(match.index, bodyClose + 1),
      syntax: syntax.slice(match.index, bodyClose + 1),
    }];
  }).filter(({ start, bodyClose }) => offset >= start && offset <= bodyClose)
    .sort((left, right) => (left.bodyClose - left.start) - (right.bodyClose - right.start));
  return functions[0];
};

const rustBraceDepthAt = (
  syntax: string,
  start: number,
  offset: number,
) => {
  let depth = 0;
  for (let index = start; index < offset; index += 1) {
    if (syntax[index] === "{") depth += 1;
    else if (syntax[index] === "}") depth -= 1;
  }
  return depth;
};

const assertCanonicalFixtureLoop = (source: string) => {
  const syntax = maskRustNonCode(source);
  const loopPattern =
    /for\s*\(\s*_,\s*sql\s*\)\s*in\s*ANALYSIS_TEST_MIGRATIONS\s*\{/g;
  if (occurrences(syntax, loopPattern) !== 1) {
    throw new Error("expected exactly one canonical migration loop");
  }
  if (occurrences(syntax, /sqlx::raw_sql\s*\(\s*sql\s*\)/g) !== 1) {
    throw new Error("expected exactly one raw_sql(sql) application");
  }
  const loopMatch = loopPattern.exec(syntax);
  if (!loopMatch || loopMatch.index === undefined) {
    throw new Error("canonical migration loop has no source location");
  }
  const owner = rustFunctionContaining(source, loopMatch.index);
  if (!owner) {
    throw new Error("canonical migration loop must be inside one function");
  }
  if (
    owner.name !== "analysis_test_pool"
    || occurrences(syntax, /\bfn\s+analysis_test_pool\s*\(/g) !== 1
  ) {
    throw new Error(
      "canonical migration loop must have exactly one enclosing analysis_test_pool function",
    );
  }
  const beginPattern =
    /\blet\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?::[^=;]+)?=\s*([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*begin\s*\(\s*\)\s*\.await\s*(?:\?|\.expect\s*\([^)]*\))\s*;/g;
  const begins = [...owner.syntax.matchAll(beginPattern)];
  if (begins.length !== 1 || begins[0].index === undefined) {
    throw new Error("canonical migration owner must begin exactly one transaction");
  }
  const transaction = begins[0][1];
  const escapedTransaction = transaction.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const applicationPattern = new RegExp(
    `sqlx::raw_sql\\s*\\(\\s*sql\\s*\\)\\s*\\.execute\\s*\\(\\s*&mut\\s+\\*${escapedTransaction}\\s*\\)\\s*\\.await\\s*(?:\\?|\\.expect\\s*\\([^)]*\\))\\s*;`,
    "g",
  );
  const commitPattern = new RegExp(
    `\\b${escapedTransaction}\\s*\\.\\s*commit\\s*\\(\\s*\\)\\s*\\.await\\s*(?:\\?|\\.expect\\s*\\([^)]*\\))\\s*;`,
    "g",
  );
  const loopOpen = syntax.indexOf("{", loopMatch.index);
  const loopClose = matchingDelimiter(syntax, loopOpen, "{", "}");
  const rawSqlIndex = syntax.search(/sqlx::raw_sql\s*\(\s*sql\s*\)/);
  if (rawSqlIndex <= loopOpen || rawSqlIndex >= loopClose) {
    throw new Error("raw_sql(sql) must be inside the canonical migration loop");
  }
  const executableLoopBody = syntax.slice(loopOpen + 1, loopClose);
  const applications = [...executableLoopBody.matchAll(applicationPattern)];
  if (applications.length !== 1 || applications[0].index === undefined) {
    throw new Error(
      "canonical migration application must execute on &mut *transaction and propagate failure",
    );
  }
  const application = applications[0];
  const applicationStart = loopOpen + 1 + application.index;
  const originalApplication = source.slice(
    applicationStart,
    applicationStart + application[0].length,
  );
  if (
    /\.expect\s*\(/.test(application[0])
    && !/\.expect\s*\(\s*"[^"]+"\s*\)/.test(originalApplication)
  ) {
    throw new Error("canonical migration application expect message must be nonempty");
  }
  const residue = executableLoopBody.split("");
  for (
    let index = application.index;
    index < application.index + application[0].length;
    index += 1
  ) {
    if (residue[index] !== "\n") residue[index] = " ";
  }
  if (residue.join("").trim() !== "") {
    throw new Error(
      "canonical migration loop body must contain only the awaited application chain",
    );
  }
  const commits = [...owner.syntax.matchAll(commitPattern)];
  if (commits.length !== 1 || commits[0].index === undefined) {
    throw new Error("canonical migration owner must commit the application transaction");
  }
  const beginIndex = owner.start + begins[0].index;
  const commitIndex = owner.start + commits[0].index;
  const applicationIndex = applicationStart;
  if (
    rustBraceDepthAt(syntax, owner.bodyOpen + 1, beginIndex) !== 0
    || rustBraceDepthAt(syntax, owner.bodyOpen + 1, loopMatch.index) !== 0
    || rustBraceDepthAt(syntax, owner.bodyOpen + 1, commitIndex) !== 0
  ) {
    throw new Error(
      "canonical begin, loop, and commit must be direct eager top-level statements",
    );
  }
  if (
    beginIndex >= loopMatch.index
    || applicationIndex <= loopMatch.index
    || applicationIndex >= loopClose
    || commitIndex <= loopClose
  ) {
    throw new Error("canonical begin, loop, application, and commit order is invalid");
  }
};

const canonicalFixtureOwner = (
  loopBody: string,
  afterLoop = "",
) => `
async fn analysis_test_pool() -> Result<()> {
    let pool = connect().await?;
    let mut transaction = pool.begin().await?;
    for (_, sql) in ANALYSIS_TEST_MIGRATIONS {
        ${loopBody}
    }
    ${afterLoop}
    transaction.commit().await?;
    Ok(())
}`;

describe("analysis migration fixture contract", () => {
  it("rejects duplicate and malformed migration fixture syntax", () => {
    const valid = `
const ANALYSIS_TEST_MIGRATIONS: [(&str, &str); 2] = [
    ("src-tauri/migrations/one.sql", include_str!("../../migrations/one.sql")),
    ("src-tauri/migrations/two.sql", include_str!("../../migrations/two.sql")),
];`;
    expect(parseFixtureMigrations(valid)).toEqual([
      {
        repositoryPath: "src-tauri/migrations/one.sql",
        includePath: "../../migrations/one.sql",
      },
      {
        repositoryPath: "src-tauri/migrations/two.sql",
        includePath: "../../migrations/two.sql",
      },
    ]);
    expect(() =>
      parseFixtureMigrations(valid.replace("two.sql", "one.sql")),
    ).toThrow("duplicate repository path");
    expect(() =>
      parseFixtureMigrations(valid.replace(
        '("src-tauri/migrations/two.sql", include_str!("../../migrations/two.sql")),',
        "unexpected_token,",
      )),
    ).toThrow("fixture declares 2 migrations but parsed 1");
    expect(() =>
      parseFixtureMigrations(
        `${valid}\nconst EXTRA_SQL: &str = include_str!("../../migrations/extra.sql");`,
      ),
    ).toThrow("unexpected include_str! outside ANALYSIS_TEST_MIGRATIONS");
    const application =
      "sqlx::raw_sql(sql).execute(&mut *transaction).await?;";
    const validLoop = canonicalFixtureOwner(application);
    expect(() => assertCanonicalFixtureLoop(validLoop)).not.toThrow();
    expect(() =>
      assertCanonicalFixtureLoop(canonicalFixtureOwner("", application)),
    ).toThrow("raw_sql(sql) must be inside the canonical migration loop");
    expect(() =>
      parseFixtureMigrations(
        valid.replace(
          "const ANALYSIS_TEST_MIGRATIONS",
          "pub(crate) const ANALYSIS_TEST_MIGRATIONS",
        ),
      ),
    ).toThrow("ANALYSIS_TEST_MIGRATIONS must remain private");
  });

  it("requires canonical migration execution on the borrowed transaction with propagation", () => {
    const application =
      "sqlx::raw_sql(sql).execute(&mut *transaction).await?;";
    const validLoop = canonicalFixtureOwner(application);
    expect(() => assertCanonicalFixtureLoop(validLoop)).not.toThrow();
    expect(() =>
      assertCanonicalFixtureLoop(
        validLoop.replace(".await?;", '.await.expect("apply");'),
      ),
    ).not.toThrow();

    for (const invalidLoop of [
      validLoop.replace(application, application.replace("&mut *transaction", "pool")),
      validLoop.replace(application, application.replace(".await?", "?")),
      validLoop.replace(application, application.replace(".await?;", ".await;")),
    ]) {
      expect(() => assertCanonicalFixtureLoop(invalidLoop)).toThrow(
        "canonical migration application must execute on &mut *transaction and propagate failure",
      );
    }
  });

  it("pins canonical migration control flow to one transaction-owning function", () => {
    const valid = `
async fn analysis_test_pool() -> Result<SqlitePool> {
    let pool = connect().await.expect("connect");
    let mut transaction = pool.begin().await.expect("begin");
    for (_, sql) in ANALYSIS_TEST_MIGRATIONS {
        sqlx::raw_sql(sql)
            .execute(&mut *transaction)
            .await
            .expect("apply");
    }
    transaction.commit().await.expect("commit");
    Ok(pool)
}`;
    expect(() => assertCanonicalFixtureLoop(valid)).not.toThrow();

    const application = `sqlx::raw_sql(sql)
            .execute(&mut *transaction)
            .await
            .expect("apply");`;
    for (const [label, bypass] of [
      ["conditional application", valid.replace(application, `if enabled {\n        ${application}\n        }`)],
      ["conditional continue", valid.replace(application, `if skip { continue; }\n        ${application}`)],
      ["early break", valid.replace(application, `break;\n        ${application}`)],
      ["early return", valid.replace(application, `return Err(error);\n        ${application}`)],
      ["different transaction binding", valid.replace(
        "let mut transaction = pool.begin().await.expect(\"begin\");",
        "let mut owner = pool.begin().await.expect(\"begin\");",
      )],
      ["begin in another function", `async fn begin_elsewhere() {
    let mut transaction = pool.begin().await.expect("begin");
}
${valid.replace(
        "    let mut transaction = pool.begin().await.expect(\"begin\");\n",
        "",
      )}`],
    ] as const) {
      expect(() => assertCanonicalFixtureLoop(bypass), label).toThrow();
    }
  });

  it("uses executable syntax and the exact analysis_test_pool owner", () => {
    const application =
      'sqlx::raw_sql(sql).execute(&mut *transaction).await.expect("apply");';
    const valid = canonicalFixtureOwner(application).replace(
      "fixture_owner",
      "analysis_test_pool",
    );
    const decoys = `
// for (_, sql) in ANALYSIS_TEST_MIGRATIONS { sqlx::raw_sql(sql); }
const NORMAL_DECOY: &str =
    "fn analysis_test_pool() { transaction.commit().await; }";
const RAW_DECOY: &str = r#"
    for (_, sql) in ANALYSIS_TEST_MIGRATIONS {
        sqlx::raw_sql(sql).execute(&mut *transaction).await?;
    }
"#;
`;
    expect(() => assertCanonicalFixtureLoop(`${decoys}${valid}`)).not.toThrow();
    expect(() =>
      assertCanonicalFixtureLoop(valid.replace("analysis_test_pool", "fixture_owner")),
    ).toThrow(/analysis_test_pool/);
  });

  it("requires the canonical loop body to be only the awaited application chain", () => {
    const application =
      'sqlx::raw_sql(sql).execute(&mut *transaction).await.expect("apply");';
    const valid = canonicalFixtureOwner(application).replace(
      "fixture_owner",
      "analysis_test_pool",
    );
    for (const [label, body] of [
      ["unpolled async", `async move { ${application} };`],
      ["closure", `let apply = || { ${application} }; apply();`],
      ["then combinator", `ready(()).then(|_| async { ${application} });`],
      ["decoy helper", `fn apply() { ${application} }`],
      ["extra statement", `let skipped = false; ${application}`],
    ] as const) {
      expect(
        () => assertCanonicalFixtureLoop(
          valid.replace(application, body),
        ),
        label,
      ).toThrow(/only the awaited application chain/);
    }
  });

  it("requires migration transaction steps to be direct eager owner statements", () => {
    const wrapped = `
async fn analysis_test_pool() -> Result<()> {
    let pool = connect().await?;
    let _pending = async {
        let mut transaction = pool.begin().await?;
        for (_, sql) in ANALYSIS_TEST_MIGRATIONS {
            sqlx::raw_sql(sql)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok::<(), sqlx::Error>(())
    };
    Ok(())
}`;

    expect(() => assertCanonicalFixtureLoop(wrapped)).toThrow(
      /direct eager top-level statements/,
    );
  });

  it("parses the non-Apalis registry prefix fail closed", () => {
    const valid = `
const FIRST_SQL: &str = include_str!("../migrations/one.sql");
const SECOND_SQL: &str =
    include_str!("../migrations/two.sql");
fn first_migration() -> Migration {
    Migration {
        sql: FIRST_SQL,
    }
}
fn second_migration() -> Migration {
    Migration {
        sql: SECOND_SQL,
    }
}
pub fn build_migrations() -> Vec<Migration> {
    let mut migrations = vec![
        first_migration(),
        second_migration(),
    ];
    migrations.extend(apalis_sqlite_migrations());
    migrations
}`;
    expect(parseMigrationRegistry(valid)).toEqual([
      "../migrations/one.sql",
      "../migrations/two.sql",
    ]);
    expect(() =>
      parseMigrationRegistry(valid.replace(
        "        second_migration(),",
        "        second_migration(),\n        second_migration(),",
      )),
    ).toThrow("duplicate migration call");
    expect(() =>
      parseMigrationRegistry(valid.replace(
        "        second_migration(),",
        "        if enabled { second_migration() },",
      )),
    ).toThrow("unmatched token");
    expect(() =>
      parseMigrationRegistry(valid.replace(
        "pub fn build_migrations() -> Vec<Migration> {\n    let mut migrations",
        "pub fn build_migrations() -> Vec<Migration> {\n    if enabled { return vec![alternate_migration()]; }\n    let mut migrations",
      )),
    ).toThrow("unmatched token before the migration vector");
    expect(() =>
      parseMigrationRegistry(
        `${valid}
pub fn build_migrations() -> Vec<Migration> {
    let mut migrations = vec![first_migration()];
    migrations.extend(apalis_sqlite_migrations());
    migrations
}`,
      ),
    ).toThrow("expected exactly one build_migrations() registry");
  });

  it("requires exactly one analysis fixture owner", () => {
    const fixtureOwners = fixtureCandidates.filter((candidate) =>
      existsSync(path.join(repositoryRoot, candidate)),
    );
    expect(
      fixtureOwners,
      `expected exactly one analysis fixture owner; found ${fixtureOwners.length}; candidates: ${fixtureCandidates.join(", ")}`,
    ).toHaveLength(1);

    const fixturePath = fixtureOwners[0];
    const fixture = read(fixturePath);
    const fixtureSyntax = maskRustNonCode(fixture);
    const registryIncludes = parseMigrationRegistry(read(migrationRegistryPath));
    const fixturePairs = parseFixtureMigrations(fixture);
    expect(registryIncludes).toHaveLength(12);
    expect(new Set(registryIncludes).size).toBe(12);
    expect(fixturePairs).toHaveLength(12);

    const registryPaths = registryIncludes.map((includePath) =>
      toRepositoryPath(
        path.resolve(
          path.dirname(path.join(repositoryRoot, migrationRegistryPath)),
          includePath,
        ),
      ),
    );
    const fixtureIncludePaths = fixturePairs.map((pair) =>
      toRepositoryPath(
        path.resolve(path.dirname(path.join(repositoryRoot, fixturePath)), pair.includePath),
      ),
    );
    const expectedIncludeRoot =
      fixturePath === fixtureCandidates[0] ? "../../migrations/" : "../../../migrations/";
    expect(fixturePairs.map((pair) => pair.repositoryPath)).toEqual(registryPaths);
    expect(fixtureIncludePaths).toEqual(registryPaths);
    expect(fixturePairs.map((pair) => pair.includePath)).toEqual(
      registryPaths.map(
        (registryPath) => `${expectedIncludeRoot}${path.posix.basename(registryPath)}`,
      ),
    );
    registryPaths.forEach((migrationPath, index) => {
      expect(migrationPath).toMatch(/^src-tauri\/migrations\//);
      expect(migrationPath).not.toContain("apalis");
      expect(path.posix.basename(migrationPath)).toMatch(
        new RegExp(`^${String(index + 1).padStart(4, "0")}_`),
      );
      expect(existsSync(path.join(repositoryRoot, migrationPath)), migrationPath).toBe(true);
    });

    expect(occurrences(fixtureSyntax, /\bANALYSIS_TEST_MIGRATIONS\b/g)).toBe(2);
    assertCanonicalFixtureLoop(fixture);
    expect(
      occurrences(
        fixtureSyntax,
        /for\s*\(\s*_,\s*sql\s*\)\s*in\s*ANALYSIS_TEST_MIGRATIONS\s*\{/g,
      ),
    ).toBe(1);
    expect(occurrences(fixtureSyntax, /sqlx::raw_sql\s*\(\s*sql\s*\)/g)).toBe(1);
    expect(occurrences(fixtureSyntax, /pool\s*\.begin\(\)\s*\.await/g)).toBe(1);
    expect(occurrences(fixtureSyntax, /transaction\s*\.commit\(\)\s*\.await/g)).toBe(1);
    expect(fixtureSyntax).not.toMatch(
      /ANALYSIS_TEST_MIGRATIONS\s*\.\s*(?:iter|into_iter|take|filter)\b/,
    );
    expect(fixtureSyntax).not.toMatch(
      /build_migrations|apply_all_migrations_for_test_pool|crate::migrations|_sqlx_migrations|\btauri(?:::|_)/,
    );

    const beginIndex = fixtureSyntax.search(/pool\s*\.begin\(\)\s*\.await/);
    const loopIndex = fixtureSyntax.search(
      /for\s*\(\s*_,\s*sql\s*\)\s*in\s*ANALYSIS_TEST_MIGRATIONS\s*\{/,
    );
    const applicationIndex = fixtureSyntax.search(/sqlx::raw_sql\s*\(\s*sql\s*\)/);
    const commitIndex = fixtureSyntax.search(/transaction\s*\.commit\(\)\s*\.await/);
    expect(beginIndex).toBeLessThan(loopIndex);
    expect(loopIndex).toBeLessThan(applicationIndex);
    expect(applicationIndex).toBeLessThan(commitIndex);

    const moduleRoot =
      fixturePath === fixtureCandidates[0]
        ? "src-tauri/src/analysis/mod.rs"
        : "src-tauri/crates/extractum-analysis/src/lib.rs";
    const moduleSource = read(moduleRoot);
    const moduleSyntax = maskRustNonCode(moduleSource);
    expect(moduleSyntax).toMatch(/#\[cfg\(test\)\]\nmod test_schema;/);
    expect(moduleSyntax).not.toMatch(/pub(?:\(crate\))?\s+mod test_schema;/);
  });
});
