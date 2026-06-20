const ANSWER_TEXT = "Mock final answer from Gemini-like page.";

function basePage(body, script = "") {
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Mock Gemini</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 24px; }
    main { display: grid; gap: 16px; max-width: 760px; }
    .composer { display: flex; gap: 8px; align-items: center; }
    [role="textbox"], textarea { min-width: 420px; min-height: 40px; }
    .answer { white-space: pre-wrap; border: 1px solid #bbb; padding: 12px; }
    .banner { border: 1px solid #a55; color: #900; padding: 12px; }
  </style>
</head>
<body>
  <main>${body}</main>
  <script>${script}</script>
</body>
</html>`;
}

const submitScript = `
const input = document.querySelector('[role="textbox"], textarea, [contenteditable="true"]');
const button = document.querySelector('[data-send]');
const answer = document.querySelector('[data-testid="assistant-answer"]');
const stop = document.querySelector('[data-testid="stop-control"]');
button?.addEventListener('click', () => {
  fetch('/mock-gemini-event', { method: 'POST', body: 'submitted' }).catch(() => undefined);
  if (stop) stop.hidden = false;
  if (input) input.setAttribute('aria-disabled', 'true');
  const chunks = ['Mock ', 'final ', 'answer ', 'from Gemini-like page.'];
  let index = 0;
  const tick = () => {
    answer.textContent += chunks[index] || '';
    index += 1;
    if (index < chunks.length) {
      setTimeout(tick, 120);
      return;
    }
    if (stop) stop.hidden = true;
    if (input) input.removeAttribute('aria-disabled');
  };
  setTimeout(tick, 120);
});
`;

const slowPauseScript = `
const input = document.querySelector('[role="textbox"]');
const button = document.querySelector('[data-send]');
const answer = document.querySelector('[data-testid="assistant-answer"]');
const stop = document.querySelector('[data-testid="stop-control"]');
button?.addEventListener('click', () => {
  stop.hidden = false;
  input.setAttribute('aria-disabled', 'true');
  answer.textContent = 'Mock ';
  setTimeout(() => { answer.textContent += 'final '; }, 1200);
  setTimeout(() => {
    answer.textContent += 'answer after pause.';
    stop.hidden = true;
    input.removeAttribute('aria-disabled');
  }, 2500);
});
`;

const neverStableScript = `
const button = document.querySelector('[data-send]');
const answer = document.querySelector('[data-testid="assistant-answer"]');
button?.addEventListener('click', () => {
  let count = 0;
  setInterval(() => {
    count += 1;
    answer.textContent = 'still generating ' + count;
  }, 150);
});
`;

export function renderMockGeminiPage(variant = "happy-path") {
  if (variant === "ready" || variant === "closed-page") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send aria-label="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "ready-missing-send") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
      </section>
    `);
  }

  if (variant === "ready-broken") {
    return basePage(`
      <section class="composer">
        <p>Gemini shell rendered, but composer controls are unavailable.</p>
      </section>
    `);
  }

  if (variant === "login-required") {
    return basePage('<section class="banner"><h1>Sign in</h1><p>Authentication is required to continue to Gemini.</p></section>');
  }

  if (variant === "captcha") {
    return basePage('<section class="banner"><h1>Verify you are human</h1><p>CAPTCHA required.</p></section>');
  }

  if (variant === "account-picker") {
    return basePage('<section class="banner"><h1>Choose an account</h1><p>Select an account to continue.</p></section>');
  }

  if (variant === "consent") {
    return basePage('<section class="banner"><h1>Before you continue</h1><p>Review privacy and terms.</p></section>');
  }

  if (variant === "rate-limit") {
    return basePage('<section class="banner"><h1>Too many requests</h1><p>Try again later.</p></section>');
  }

  if (variant === "unknown-modal") {
    return basePage('<div role="dialog" aria-label="Gemini notice"><p>Manual review required.</p></div>');
  }

  if (variant === "textarea-input") {
    return basePage(`
      <section class="composer">
        <textarea placeholder="Ask Gemini"></textarea>
        <button data-send aria-label="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "wrapped-dom") {
    return basePage(`
      <section><div><div><form class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <span role="button" data-send aria-label="Send message">Send</span>
      </form></div></div></section>
      <section><article class="answer" data-testid="assistant-answer"></article></section>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "contenteditable-input") {
    return basePage(`
      <section class="composer">
        <div contenteditable="true" role="textbox" aria-label="Message Gemini"></div>
        <button data-send title="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "icon-send") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send title="Send"><span aria-hidden="true">&uarr;</span></button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "slow-pauses") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send aria-label="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, slowPauseScript);
  }

  if (variant === "never-stable") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send aria-label="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop">Stop</button>
    `, neverStableScript);
  }

  if (variant === "broken-answer") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send aria-label="Send">Send</button>
      </section>
    `, "");
  }

  return basePage(`
    <section class="composer">
      <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
      <button data-send aria-label="Send">Send</button>
    </section>
    <article class="answer" data-testid="assistant-answer"></article>
    <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
  `, submitScript);
}

export { ANSWER_TEXT };
