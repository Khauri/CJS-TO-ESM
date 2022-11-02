# mdnzr

Uses swc to modernize your js.
Status: Experimental and still under development. Use at your own risk.

## Usage

```sh
npx mdnzr@latest [options] <glob>
```

Where `glob` is a variadic list of globs. To avoid shell expansion wrap the command in quotes, otherwise you may hit
an argument limit if you are trying to grab many files.

Run `npx mdnzr@latest -h` to see all options 

```
Options:
  -V, --version                output the version number
  -o, --output <path>          output path. If not provided, output will be written to stdout
  -i, --ignore <path>          patterns to ignore, separated by comma
  -c, --concurrency <number>   number of files that can be processed at once
  -e, --extension <extension>  output file extension
  -w, --write                  write output to file
  -h, --help                   display help for command
```