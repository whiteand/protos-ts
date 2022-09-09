#!/usr/bin/env node

const util = require('node:util');
const spawn = util.promisify(require('node:child_process').spawn);

async function main() {
    console.log('here')
    console.log('in', process.cwd())
}


main()
