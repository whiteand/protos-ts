#!/usr/bin/env node

const path = require("path");
const process = require("process");
const { spawn } = require("child_process");

async function main(parameters) {
  const cliPath = getCliPath();
  const cliProcess = spawn(cliPath, parameters, {
    cwd: process.cwd(),
  });
  const code = await new Promise(resolve => cliProcess.on('close', resolve))
  process.exit(code)
}

function getCliPath() {
  const currentOs = process.platform;

  if (currentOs === "linux") {
    return path.resolve(__dirname, "./bin/protos-ts-linux");
  }
  if (currentOs === "win32") {
    return path.resolve(__dirname, "./bin/protos-ts-win.exe");
  }
  // TODO: add support for 'aix'
  // TODO: add support for 'darwin'
  // TODO: add support for 'freebsd'
  // TODO: add support for 'linux'
  // TODO: add support for 'openbsd'
  // TODO: add support for 'sunos'
  failWithBugReport(`Sorry, unsupported OS: "${process.platform}"`);
}

function fail(message) {
  console.error(message);
  process.exit(1);
}

function failWithBugReport(message) {
  fail(`${message}\nPlease open the issue in ${require("./package.json").bugs.url}`);
}

const parameters = process.argv.slice(2);

main(parameters).catch((error) => {
    failWithBugReport(`Unexpacted error: ${error.message}`);
});
