#!/usr/bin/node
import {program} from 'commander';
import {transform} from './index.js';

program.version('0.1.0')
  .option('-o, --output <path>', 'output path. If not provided, output will be written to stdout')
  .option('-i, --ignore <path>', 'patterns to ignore, separated by comma')
  .option('-c, --concurrency <number>', 'number of files that can be processed at once')
  .option('-e, --extension <extension>', 'output file extension')
  .option('-w, --write', 'write output to file')
  .arguments('<file>', 'Global patterns of files to transform. Wrap in quotes to avoid shell expansion.')
  .parse(process.argv);

const options = program.opts();
// Call the transformer
transform({
  globs: program.args, 
  ignore: options.ignore, 
  concurrency: options.concurrency,
  outputExtension: options.extension,
  write: options.write,
});