const reset = "\x1b[0m";
const coloredLog = {
  green: (text: string) => console.log("\x1b[32m" + text + reset),
  red: (text: string) => console.log("\x1b[31m" + text + reset),
  blue: (text: string) => console.log("\x1b[34m" + text + reset),
  yellow: (text: string) => console.log("\x1b[33m" + text + reset),
};

export class Logger {
  constructor(private readonly context: string) {}

  debug(message: string) {
    coloredLog.green(`[${this.context}][DEBUG] ${message}`);
  }

  info(message: string) {
    coloredLog.blue(`[${this.context}][INFO] ${message}`);
  }

  warn(message: string) {
    coloredLog.yellow(`[${this.context}][WARN] ${message}`);
  }

  error(message: string) {
    coloredLog.red(`[${this.context}][ERROR] ${message}`);
  }
}
