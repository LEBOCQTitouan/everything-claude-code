/**
 * Shared ANSI color utilities.
 * Zero dependencies — uses raw escape codes.
 * Respects NO_COLOR (https://no-color.org) and non-TTY environments.
 */

type ColorFn = (s: string) => string;

const noColor = Boolean(process.env.NO_COLOR) ||
  (!process.stdout.isTTY && !process.stderr.isTTY);

function wrap(code: string): ColorFn {
  if (noColor) return (s: string) => s;
  return (s: string) => `\x1b[${code}m${s}\x1b[0m`;
}

// Modifiers
export const bold: ColorFn    = wrap('1');
export const dim: ColorFn     = wrap('2');

// Colors
export const red: ColorFn     = wrap('31');
export const green: ColorFn   = wrap('32');
export const yellow: ColorFn  = wrap('33');
export const cyan: ColorFn    = wrap('36');
export const white: ColorFn   = wrap('37');
export const magenta: ColorFn = wrap('35');
export const gray: ColorFn    = wrap('90');

// Backgrounds
export const bgCyan: ColorFn  = wrap('46');

/**
 * Strip all ANSI escape sequences from a string.
 */
export function stripAnsi(str: string): string {
  // eslint-disable-next-line no-control-regex
  return str.replace(/\x1b\[[0-9;]*m/g, '');
}
