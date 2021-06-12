set -e

watchexec --exts ts --ignore 'lib node_modules client' -- "npx tsc && echo '\n==reloaded==\n'"

