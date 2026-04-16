import katex from 'katex';

export function renderMath(container: HTMLElement): void {
  // Render <span data-math-style="inline">...</span>
  container.querySelectorAll('span[data-math-style]').forEach((el) => {
    const mathEl = el as HTMLElement;
    const style = mathEl.getAttribute('data-math-style');
    const latex = mathEl.textContent || '';
    try {
      katex.render(latex, mathEl, {
        displayMode: style === 'display',
        throwOnError: false,
      });
      mathEl.removeAttribute('data-math-style');
    } catch {
      mathEl.classList.add('math-error');
    }
  });

  // Render <code class="language-math" data-math-style="display">...</code>
  container.querySelectorAll('code[data-math-style]').forEach((el) => {
    const codeEl = el as HTMLElement;
    const style = codeEl.getAttribute('data-math-style');
    const latex = codeEl.textContent || '';
    const wrapper = document.createElement(style === 'display' ? 'div' : 'span');
    wrapper.className = 'math-display';
    try {
      katex.render(latex, wrapper, {
        displayMode: style === 'display',
        throwOnError: false,
      });
      // Replace the <pre><code>...</code></pre> parent with rendered output
      const pre = codeEl.parentElement;
      if (pre && pre.tagName === 'PRE') {
        pre.replaceWith(wrapper);
      } else {
        codeEl.replaceWith(wrapper);
      }
    } catch {
      codeEl.classList.add('math-error');
    }
  });
}
