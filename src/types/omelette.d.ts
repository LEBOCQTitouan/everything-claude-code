declare module 'omelette' {
  interface EventArgs {
    reply: (items: string[]) => void;
    before: string;
  }

  interface Completion {
    on(event: string, handler: (args: EventArgs) => void): void;
    init(): void;
    setupShellInitFile(): void;
  }

  function omelette(template: string): Completion;
  export = omelette;
}
