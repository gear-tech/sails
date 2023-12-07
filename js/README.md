## Installation
The package can be installed as either a global dependency or a package dependency.

```bash
npm install -g sails-js
```
or
```bash
npm install sails-js
```

## Usage

### CLI
- Generate typescript code from the IDL file
```bash
sails-js generate path/to/sails.idl -o path/to/out/dir
```
This command will generate 2 files to the specified directory.

- Parse IDL file and print the result
```bash
sails-js parse-and-print path/to/sails.idl
```

- Parse IDL file and save the result to a json file
```bash
sails-js parse-into-file path/to/sails.idl path/to/out.json
```
