import swc from '@swc/core';
import {globbySync} from 'globby';
import fs from 'node:fs';
import path from 'node:path';

export async function transform({
  globs,
  ignore,
  write = false,
  outputExtension, 
  concurrency = 15, // number of files that can be processed at once 
} = {}) {
  const files = globbySync(globs, {onlyFiles: true, ignore: ignore?.split?.(',') ?? []});
  // Maybe split the work into multiple worker threads at some point? swc is pretty fast though so maybe 
  // we should just limit the number of files processed at once.
  while(files.length) {
    const chunk = files.splice(0, concurrency);
    await Promise.all(chunk.map(async (file) => {
      const filename = path.basename(file);
      const newFilename = outputExtension ? filename.replace(path.extname(filename), outputExtension) : filename;
      const outputFile = path.join(path.dirname(file), newFilename);
      const {code} = swc.transformFileSync(file, {
        sourceMaps: false,
        jsc: {
          preserveAllComments: true,
          target: 'es2019',
          experimental: {
            plugins: [
              ['swc-plugin-cjs-to-esm', {}]
            ]
          }
        },
        swcrc: false,
      });
      if(write) {
        fs.writeFileSync(outputFile, code);
      } else {
        console.log(code);
      }
    }));
  }
}